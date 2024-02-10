// Parser module

// Syntax tree structs
pub mod syntax_tree;

use crate::scanner::{tokens::*, Scanner};
use std::cell::RefCell;
use std::rc::Rc;
use syntax_tree::*;

pub struct Parser {
    // Tokens stream
    scanner: Scanner,
    // currently (`processed` or `to be processed`) token
    current: Option<Token>,
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

    // AFTER PARSING AN ARBITRARY EXPRESSION, FIELD `current` MUST
    // POINT TO THE VERY FIRST CHARACTER (IF ANY, OR EmptyString token)
    // AFTER THE MOST RECENTLY PARSED EXPRESSION
    // This rule will be `coded` soon
    // Regexp => EmptyString | Union
    fn parse_expression(&mut self) -> Result<Option<Rc<RefCell<Regexp>>>, String> {
        // Attempt to parse an empty expression
        // or `a union expression as denoted by grammar rule above` -- coming soon
        match &self.current {
            Some(peek) => {
                // There are tokens to be processed.
                match peek.name {
                    TokenName::EmptyString => {
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
                    // Attempt to parse a group expression
                    TokenName::LeftParen => self.parse_group(),
                    // Placeholder code
                    _ => Ok(None),
                }
            }
            _ => {
                // end of stream, no more tokens to process
                Ok(None) // no expression was parsed
            }
        }
    }

    fn parse_group(&mut self) -> Result<Option<Rc<RefCell<Regexp>>>, String> {
        // Move past opening (
        self.advance();
        // parse an arbitrary expression or report error (? operator)
        match self.parse_expression()? {
            Some(parsed_expression) => {
                // `parsed_expression` has type Rc<RefCell<Regexp>>

                // Advance only when current item has name TokenName::RightParent
                // or report error `Expected )` (? operator)
                self.consume(TokenName::RightParen, "Expected )")?;
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
                let error = "Expected expression after (";
                let error_position = self.current.as_ref().unwrap().position;
                let mut error_indicator = String::with_capacity(error_position);
                while error_indicator.len() < error_position {
                    error_indicator.push(' ');
                }
                error_indicator.push('^');
                let source = self.scanner.get_source_string();
                Err(format!(
                    "Syntax error in position {error_position}: {error}\n\
                    {source}\n{error_indicator}"
                ))
            }
        }
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
            let error_position = self.current.as_ref().unwrap().position;
            let mut error_indicator = String::with_capacity(error_position);
            while error_indicator.len() < error_position {
                error_indicator.push(' ');
            }
            error_indicator.push('^');
            let source = self.scanner.get_source_string();
            return Err(format!(
                "Syntax error in position {error_position}: {error}\n\
                {source}\n{error_indicator}"
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
