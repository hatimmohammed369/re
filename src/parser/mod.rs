// Parser module

// Syntax tree structs
pub mod syntax_tree;

use std::cell::RefCell;
use crate::scanner::{Scanner, tokens::*};
use syntax_tree::*;

pub struct Parser {
    // Tokens stream
    scanner: Scanner,
    // currently (`processed` or `to be processed`) token
    current: Option<Token>,
}

impl Parser {
    pub fn new(source: &str) -> Parser {
        let scanner = Scanner::new(source);
        let current = None;
        Parser { scanner, current }
    }

    fn advance(&mut self) {
        // read next token in stream
        self.current = self.scanner.next();
    }

    fn parse(&mut self) -> Regexp {
        // Parse an EmptyExpression
        self.advance(); // grab the first token in stream
        match &self.current {
            Some(peek) => {
                // There are tokens to be processed.
                match peek.name {
                    TokenName::EmptyString => {
                        let tag = ExpressionTag::EmptyExpression;
                        let parent = None;
                        let children = RefCell::new(vec![]);
                        Regexp { tag, parent, children }
                    }
                    _ => {
                        // a placeholder code
                        panic!()
                    }
                }
            }
            _ => {
                // end of stream, no more tokens to process
                panic!()
            }
        }
    }
}

pub fn parse(source: &str) -> Regexp {
    // parse source string into a `Regexp` object
    Parser::new(source).parse()
}
