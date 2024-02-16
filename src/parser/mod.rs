// Parser module
// Take Tokens generated by the scanner and structure them

// Syntax tree structs
pub mod syntax_tree;

use crate::format_error;
use crate::scanner::{tokens::*, Scanner};
use std::cell::RefCell;
use std::rc::Rc;
use syntax_tree::*;

#[allow(dead_code)]
// Mark where to a grouping begins
enum GroupingMark {
    // There is a ( in index `position` in source string
    Group { position: usize },
}

pub struct Parser {
    // Tokens stream
    scanner: Scanner,
    // currently (`processed` or `to be processed`) token
    current: Option<Token>,
    // -------------------- NOTE --------------------
    // AFTER PARSING AN ARBITRARY EXPRESSION, FIELD `current` MUST
    // POINT TO THE VERY FIRST CHARACTER (IF ANY, OR Empty token)
    // AFTER THE MOST RECENTLY PARSED EXPRESSION
    // ----------------------------------------------

    // marks stack
    // we need a stack because groups (...) can nest
    grouping_marks: Vec<GroupingMark>,
}

impl Parser {
    fn new(source: &str) -> Parser {
        let scanner = Scanner::new(source);
        let current = None;
        let grouping_marks = vec![];
        Parser {
            scanner,
            current,
            grouping_marks,
        }
    }

    // Attempt to parse source string
    fn parse_source(&mut self) -> Result<Regexp, String> {
        // Grab the first token in stream
        self.advance()?;
        match self.parse_expression() {
            // A syntax error occurred while parsing source string
            Err(error) => Err(error),

            // Successfully parsed source string
            Ok(option_regexp) => {
                // `option_regexp` has type Option<Rc<RefCell<Regexp>>>
                match option_regexp {
                    Some(regexp) => {
                        // When done parsing, take the resulting `Regexp`
                        // and clone (shallow copy) its data
                        // - Field `tag` holds data local to expression (namely its type)
                        // and also it's easily clone
                        // - Fields `parent` and `children` hold counted references
                        // which are easily copied without losing referenced data
                        // those counted references only increase their internal count when cloned
                        let tag = regexp.borrow().tag.clone();
                        // parent is `None` because this `Regexp` is syntax tree root.
                        let parent = None;
                        // Clone Rc's which is merely increasing internal reference count
                        let children = RefCell::new(
                            regexp
                                .borrow()
                                .children
                                .borrow()
                                .iter()
                                .map(Rc::clone)
                                .collect::<Vec<_>>(),
                        );

                        // Successfully parsed entire source string
                        Ok(Regexp {
                            tag,
                            parent,
                            children,
                        })
                    }
                    None => {
                        // Could not parse source string for some unknown reason
                        // maybe a bug in code
                        // Because even an empty source string has at least one
                        // token, namely Empty, thus we can parse a Regexp
                        // with its `tag` field set to ExpressionTag::EmptyExpression
                        eprintln!("FATAL ERROR:");
                        eprintln!(
                            "Could not parse source string `{}`\n",
                            self.scanner.get_source_string()
                        );
                        panic!()
                    }
                }
            }
        }
    }

    // Regexp => Concatenation ( "|" Regexp )?
    fn parse_expression(&mut self) -> Result<Option<Rc<RefCell<Regexp>>>, String> {
        match &self.current {
            None => {
                // Reached end of input, no expression can be parsed
                Ok(None)
            }
            Some(token) => {
                // There are unprocessed tokens
                match token.name {
                    // This token can begin a valid expression
                    TokenName::Empty
                    | TokenName::Dot { .. }
                    | TokenName::Character { .. }
                    | TokenName::LeftParen => {
                        // Attempt to parse an arbitrary expression
                        // But do that attempt to parse an alternation expression
                        // because alternation has the lowest precedence of all regular expressions operations
                        let alternation = Regexp::new(ExpressionTag::Alternation);

                        // First, attempt to parse one concatenation
                        if let Some(concatenation) = self.parse_concatenation()? {
                            alternation.children.borrow_mut().push(concatenation);

                            // Parse another alternation only if current token is |
                            if self.check(TokenName::Pipe) {
                                // Move past current |
                                self.advance()?;
                                if let Some(expression) = self.parse_expression()? {
                                    // Parsed a new expression
                                    // append it to field `children` of this `alternation`
                                    alternation.children.borrow_mut().push(expression);
                                }
                            }
                        }

                        // Can't use `alternation.children.borrow().len()` directly with `match`
                        // because `alternation` is moved inside `match` body
                        let parsed_expressions = alternation.children.borrow().len();
                        match parsed_expressions {
                            0 => {
                                // No expression was parsed, possibly end of pattern
                                Ok(None)
                            }
                            1 => {
                                // One expression was parsed, but alternation expressions are composed
                                // of at least two expressions, thus it makes no sense to return this single
                                // expression as an alternation
                                // Return this expression verbatim
                                Ok(alternation.children.borrow_mut().pop())
                            }
                            _ => {
                                // At least two expressions were parsed
                                // Composed an alternation expression
                                // Its children are already inside it, in Regexp field `children`
                                let alternation = Rc::new(RefCell::new(alternation));
                                alternation
                                    .borrow_mut()
                                    .children
                                    .borrow_mut()
                                    .iter_mut()
                                    .for_each(|child| {
                                        // Make each child obtain a weak reference to its parent `alternation`
                                        child.borrow_mut().parent =
                                            Some(Rc::downgrade(&alternation));
                                    });

                                // Successfully parsed an alternation expression
                                Ok(Some(alternation))
                            }
                        }
                    }
                    _ => {
                        // Any token which can not begin a valid expression, like + or *
                        let source = self.scanner.get_source_string();
                        let error_char = &source[token.position..=token.position];
                        let error = format!("Expected expression before {error_char}");
                        let (error_index, error_position) = {
                            match &self.current {
                                Some(Token { position, .. }) => {
                                    (*position, format!("in position {position}"))
                                }
                                None => (source.len(), String::from("at end of pattern")), // in case parser reached end of input
                            }
                        };
                        Err(format_error(
                            &format!("Syntax error {error_position}: {error}"),
                            &source,
                            &[(error_index, 1_u8)],
                            "",
                        ))
                    }
                }
            }
        }
    }

    // Concatenation => Primary+
    fn parse_concatenation(&mut self) -> Result<Option<Rc<RefCell<Regexp>>>, String> {
        // Attempt to parse a concatenation of regular expressions

        let concatenation = Regexp::new(ExpressionTag::Concatenation);
        while let Some(primary_expression) = self.parse_primary()? {
            // Parsed a new expression
            // append it to field `children` of this `alternation`
            concatenation.children.borrow_mut().push(primary_expression);
        }

        // Can't use `concatenation.children.borrow().len()` directly with `match`
        // because `concatenation` is moved inside `match` body
        let parsed_expressions = concatenation.children.borrow().len();
        match parsed_expressions {
            0 => {
                // No expression was parsed, possibly end of pattern
                Ok(None)
            }
            1 => {
                // One expression was parsed, but concatenation expressions are composed
                // of at least two expressions, thus it makes no sense to return this single
                // expression as a concatenation
                // Return this expression verbatim
                Ok(concatenation.children.borrow_mut().pop())
            }
            _ => {
                // At least two expressions were parsed
                // Composed a concatenation expression
                // Its children are already inside it, in Regexp field `children`
                let concatenation = Rc::new(RefCell::new(concatenation));
                concatenation
                    .borrow_mut()
                    .children
                    .borrow_mut()
                    .iter_mut()
                    .for_each(|child| {
                        // Make each child obtain a weak reference to its parent `concatenation`
                        child.borrow_mut().parent = Some(Rc::downgrade(&concatenation));
                    });

                // Successfully parsed a concatenation expression
                Ok(Some(concatenation))
            }
        }
    }

    // Primary => Empty | Group | MatchCharacter | MatchAnyCharacter
    fn parse_primary(&mut self) -> Result<Option<Rc<RefCell<Regexp>>>, String> {
        // WHAT DO YOU DO `parse_primary`?
        // I parse primary expressions, which are:
        // - The empty regular expression
        // - The dot expression `.`
        // - Character expressions like `x`
        // - Grouped regular expressions, like `(abc)`

        match &self.current.as_ref() {
            Some(token) => {
                match &token.name {
                    TokenName::Empty => self.parse_the_empty_expression(),
                    TokenName::Dot => self.parse_the_dot_expression(),
                    TokenName::Character { value, .. } => self.parse_character_expression(*value),
                    TokenName::LeftParen => self.parse_group(),
                    _ => Ok(None), // Current token can begin a valid expression
                }
            }
            None => Ok(None), // End of pattern
        }
    }

    // Group => "(" Regexp ")"
    fn parse_group(&mut self) -> Result<Option<Rc<RefCell<Regexp>>>, String> {
        // Attempt to:
        // First : parse an arbitrary expression
        // Second: After `First` is finished, search for a )
        // If either `First` or `Second` fails, report an error as follow:
        // `First` failed : report error `Expected expression after (`
        // `Second` failed: report error `Expected ) after expression`
        // These rules are due to grammar rule: Group => "(" Regexp ")"
        // First : After `(` parser expects a `Regexp`
        // Second: After `Regexp` parser expects a `)`

        // Move past opening (
        self.advance()?;

        // parse an arbitrary expression or report error (? operator)
        match self.parse_expression()? {
            Some(parsed_expression) => {
                // `parsed_expression` has type Rc<RefCell<Regexp>>

                // Advance only when current item has name TokenName::RightParent
                // or report error `Expected ) after expression` (? operator)
                self.consume(TokenName::RightParen, "Expected ) after expression")?;
                // field `current` now points to the first character (or Empty token)
                // after the closing )

                // Construct parsed grouped expression
                let group = Regexp::new(ExpressionTag::Group {
                    quantifier: self.consume_quantifier()?,
                });
                // let `group` take ownership of the expression it encloses
                group.children.borrow_mut().push(parsed_expression);
                // convert `group` to appropriate return type
                let group = Rc::new(RefCell::new(group));
                // make enclosed expression `parent` field points to `group`
                group.borrow_mut().children.borrow_mut()[0]
                    .borrow_mut()
                    .parent = Some(Rc::downgrade(&group));

                // Successfully parsed a grouped expression
                Ok(Some(group))
            }
            None => {
                // Syntax error: Expected expression after (
                // But why? parser call `parse_group` only when
                // its field `current` has type (field `name` in struct Token) is
                // `TokenName::LeftParen`
                // In other words, what the parser currently process is a (
                // it makes sense to attempt to parse a grouped expression
                // because that's what the grammar rule `Group => "(" Regexp ")"` says
                // So when the parser follows what the grammar says and fails
                // it's a syntax error you made
                let error = "Expected expression after (";
                let source = self.scanner.get_source_string();
                let (error_index, error_position) = {
                    match &self.current {
                        Some(Token { position, .. }) => {
                            (*position, format!("in position {position}"))
                        }
                        None => (source.len(), String::from("at end of pattern")), // in case parser reached end of input
                    }
                };
                Err(format_error(
                    &format!("Syntax error {error_position}: {error}"),
                    &source,
                    // Place one (1_u8) caret `^` below error position
                    // in source string as a visual aid
                    &[(error_index, 1_u8)],
                    "", // Hints
                ))
            }
        }
    }

    // Empty => ""
    fn parse_the_empty_expression(&mut self) -> Result<Option<Rc<RefCell<Regexp>>>, String> {
        // Move past Empty token
        self.advance()?;
        // field `current` now points to the first character after
        // the position of Empty we had before the above call
        // to `advance`. Note that it can not point to another
        // Empty token because the scanner never generates
        // two or more Empty tokens in row

        // Successfully parsed an empty expression
        Ok(Some(Rc::new(RefCell::new(Regexp::new(
            ExpressionTag::EmptyExpression,
        )))))
    }

    // MatchAnyCharacter => Dot
    fn parse_the_dot_expression(&mut self) -> Result<Option<Rc<RefCell<Regexp>>>, String> {
        // Move past Dot token
        self.advance()?;

        // Successfully parsed a dot expression
        Ok(Some(Rc::new(RefCell::new(Regexp::new(
            ExpressionTag::DotExpression {
                quantifier: self.consume_quantifier()?,
            },
        )))))
    }

    // Character => OrdinaryCharacter | EscapedMetacharacter
    fn parse_character_expression(
        &mut self,
        value: char,
    ) -> Result<Option<Rc<RefCell<Regexp>>>, String> {
        // Move past `Character` token
        self.advance()?;

        // Successfully parsed a character expression
        Ok(Some(Rc::new(RefCell::new(Regexp::new(
            ExpressionTag::CharacterExpression {
                value,
                quantifier: self.consume_quantifier()?,
            },
        )))))
    }

    // Read next token in stream
    fn advance(&mut self) -> Result<(), String> {
        self.current = self.scanner.next();
        if self.check(TokenName::RightParen) && self.grouping_marks.pop().is_none() {
            // There is no group expression currently processed
            // Thus ) was used without its matching (
            // Syntax error!
            let error = "Unbalanced )\n) is used without a matching (";
            let source = self.scanner.get_source_string();
            let (error_index, error_position) = {
                match &self.current {
                    Some(Token { position, .. }) => (*position, format!("in position {position}")),
                    None => (source.len(), String::from("at end of pattern")), // in case parser reached end of input
                }
            };
            return Err(format_error(
                &format!("Syntax error {error_position}: {error}"),
                &source,
                // Place one (1_u8) caret `^` below error position
                // in source string as a visual aid
                &[(error_index, 1_u8)],
                // Hints
                "\nTo match a literal ) use \\)\n\
                To match a metacharacter, precede it with a slash in your pattern \\\n\
                To match a *, for instance, use \\* in your pattern\n\n\
                But remember, \\\\ inside your (rust non-raw string) pattern is one slash for the regular expressions engine\n\
                Hence to match a single literal slash, you write pattern \\\\\\\\\n\
                The first pair (one slash, operator) escape the second pair (one slash, operand)\n\
                Or, you can use a raw string r\"\\\\\"",
            ));
        } else if self.check(TokenName::LeftParen) {
            // The parser has found a possibly opening (
            // Note the word `possibly`, if pattern ends with a matching )
            // then the parser will report a syntax error
            self.grouping_marks.push(GroupingMark::Group {
                position: self.current.as_ref().unwrap().position,
            });
            return Ok(());
        }

        Ok(())
    }

    // Check if current token (if any) has a given type
    fn check(&self, expected: TokenName) -> bool {
        match &self.current {
            Some(token) => token.name == expected,
            None => false,
        }
    }

    // Check if current token (if any) has a given type
    // if true then advance
    // if false report `error`
    fn consume(&mut self, expected: TokenName, error: &str) -> Result<(), String> {
        if !self.check(expected) {
            // current token name (type) is not what was expected
            // in other words, grammar requires a specific item to appear here
            // but parser found something else
            // this is a syntax error
            let source = self.scanner.get_source_string();
            let (error_index, error_position) = {
                match &self.current {
                    Some(Token { position, .. }) => (*position, format!("in position {position}")),
                    None => (source.len(), String::from("at end of pattern")), // in case parser reached end of input
                }
            };
            return Err(format_error(
                &format!("Syntax error {error_position}: {error}"),
                &self.scanner.get_source_string(),
                // Place one (1_u8) caret `^` below error position
                // in source string as a visual aid
                &[(error_index, 1_u8)],
                "", // Hints
            ));
        }
        self.advance()?;
        Ok(())
    }

    fn consume_quantifier(&mut self) -> Result<Quantifier, String> {
        // Check current token, if its name (field `name`) is either one of:
        // Mark, Star, Plus
        // Consume each and construct a Quantifier variant
        let quantifier = Quantifier::from(&self.current);
        if !matches!(quantifier, Quantifier::None) {
            // We found a quantifier, consume it
            self.advance()?;
        }
        Ok(quantifier)
    }
}

// A public interface function
pub fn parse(source: &str) -> Result<Regexp, String> {
    // parse source string into a `Regexp` object
    Parser::new(source).parse_source()
}
