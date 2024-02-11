// Parser module

// Syntax tree structs
pub mod syntax_tree;

use crate::format_error;
use crate::scanner::{tokens::*, Scanner};
use std::cell::RefCell;
use std::rc::Rc;
use syntax_tree::*;

pub struct Parser {
    // Tokens stream
    scanner: Scanner,
    // currently (`processed` or `to be processed`) token
    current: Option<Token>,
    // -------------------- NOTE --------------------
    // AFTER PARSING AN ARBITRARY EXPRESSION, FIELD `current` MUST
    // POINT TO THE VERY FIRST CHARACTER (IF ANY, OR EmptyString token)
    // AFTER THE MOST RECENTLY PARSED EXPRESSION
    // ----------------------------------------------
}

impl Parser {
    fn new(source: &str) -> Parser {
        let scanner = Scanner::new(source);
        let current = None;
        Parser { scanner, current }
    }

    // Attempt to parse source string
    fn parse_source(&mut self) -> Result<Regexp, String> {
        // Grab the first token in stream
        self.advance();
        match self.parse_expression() {
            Ok(option_regexp) => {
                // `option_regexp` has type Option<Rc<RefCell<Regexp>>>
                match option_regexp {
                    Some(regexp) => {
                        // When done parsing, take the resulting `Regexp`
                        // and clone (shallow copy) its data
                        // field `tag` holds data local to expression (namely its type)
                        // and also it's easily clone
                        // fields `parent` and `children` hold counted references
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
                        // token, namely EmptyString, thus we can parse a Regexp
                        // with its `tag` field set to ExpressionTag::EmptyExpression
                        let source = self.scanner.get_source_string();
                        eprintln!("Could not parse source string `{source}`");
                        panic!()
                    }
                }
            }
            // A syntax error occurred while parsing source string
            Err(error) => Err(error),
        }
    }

    // Regexp => Primary
    fn parse_expression(&mut self) -> Result<Option<Rc<RefCell<Regexp>>>, String> {
        // Attempt to parse an arbibrary expression
        if self.current.is_some() {
            // There are tokens to be processed.
            self.parse_primary()
        } else {
            // end of stream, no more tokens to process
            Ok(None) // No expression was parsed
        }
    }

    // Primary => EMPTY_STRING | Group
    fn parse_primary(&mut self) -> Result<Option<Rc<RefCell<Regexp>>>, String> {
        // WHAT DO YOU DO `parse_primary`?
        // I parse primary expressions, which are:
        // - The empty regular expression
        // - Grouped regular expressions, like (abc)

        // Note that `parse_primary` is called only when `parse_expression`
        // confirmed that we have tokens to process
        // in other words, parser field `current` is not Option::None
        // thus expression `self.current.unwrap()` can NEVER panic!

        match &self.current.as_ref().unwrap().name {
            TokenName::EmptyString => self.parse_the_empty_expression(),
            TokenName::LeftParen => self.parse_group(),
            other => {
                // Placeholder code
                eprintln!("Parser is incomplete!");
                eprintln!(
                    "Can not parse an expression beginning\
                    with a `TokenName::{other:#?}` token"
                );
                panic!();
            }
        }
    }

    // Group => "(" Regexp ")"
    fn parse_group(&mut self) -> Result<Option<Rc<RefCell<Regexp>>>, String> {
        // Attempt to:
        // First : parse an abitrary expression
        // Second: After `First` is finished, search for a )
        // If either `First` or `Second` fails, report an error as follow:
        // `First` failed : report error `Expected expression after (`
        // `Second` failed: report error `Expected ) after expression`
        // These rules are due to grammar rule: Group => "(" Regexp ")"
        // First : After `(` parser expects a `Regexp`
        // Second: After `Regexp` parser expects a `)`

        // Move past opening (
        self.advance();

        // parse an arbitrary expression or report error (? operator)
        match self.parse_expression()? {
            Some(parsed_expression) => {
                // `parsed_expression` has type Rc<RefCell<Regexp>>

                // Advance only when current item has name TokenName::RightParent
                // or report error `Expected ) after expression` (? operator)
                self.consume(TokenName::RightParen, "Expected ) after expression")?;
                // field `current` now points to the fisrt character (or EmptyString token)
                // after the closing )

                // Construct parsed grouped expression
                let group = Regexp::new(ExpressionTag::Group);
                // let `group` take ownershiped of the expression it encloses
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
                let error_position = {
                    match &self.current {
                        Some(Token { position, .. }) => *position,
                        None => source.len(), // in case parser reached end of input
                    }
                };
                Err(format_error(
                    &format!("Syntax error {}: {error}", {
                        match &self.current {
                            Some(Token { position, .. }) => format!("in position {position}"),
                            None => String::from("at end of input"), // in case parser reached end of input
                        }
                    }),
                    &source,
                    // Place one (1_u8) caret `^` below error position
                    // in source string as a visual aid
                    &[(error_position, 1_u8)],
                    "", // Hints
                ))
            }
        }
    }

    fn parse_the_empty_expression(&mut self) -> Result<Option<Rc<RefCell<Regexp>>>, String> {
        // Move past EmptyString token
        self.advance();
        // field `current` now points to the first character after
        // the position of EmptyString we had before the above call
        // to `advance`. Note that it can not point to another
        // EmptyString token because the scanner never generates
        // two or more EmptyString tokens in row

        // Successfully parsed an empty expression
        Ok(Some(Rc::new(RefCell::new(Regexp::new(
            ExpressionTag::EmptyExpression,
        )))))
    }

    // Read next token in stream
    fn advance(&mut self) {
        self.current = self.scanner.next();
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
            let error_position = {
                match &self.current {
                    Some(Token { position, .. }) => *position,
                    None => source.len(), // in case parser reached end of input
                }
            };
            return Err(format_error(
                &format!("Syntax error {}: {error}", {
                    match &self.current {
                        Some(Token { position, .. }) => format!("in position {position}"),
                        None => String::from("at end of input"), // in case parser reached end of input
                    }
                }),
                &self.scanner.get_source_string(),
                // Place one (1_u8) caret `^` below error position
                // in source string as a visual aid
                &[(error_position, 1_u8)],
                "", // Hints
            ));
        }
        self.advance();
        Ok(())
    }
}

// A public interface function
pub fn parse(source: &str) -> Result<Regexp, String> {
    // parse source string into a `Regexp` object
    Parser::new(source).parse_source()
}
