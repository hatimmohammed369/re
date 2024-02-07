#[allow(dead_code)]
pub mod tokens;

use tokens::{Token, TokenName::*};

pub struct Scanner {
    source: Vec<char>,
    current: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Scanner {
        let source = source.chars().collect::<Vec<_>>();
        let current = 0;
        Scanner { source, current }
    }

    pub fn advance(&mut self) {
        self.current += 1;
    }

    pub fn has_next(&self) -> bool {
        self.current < self.source.len()
    }

    pub fn peek(&self) -> char {
        *self.source.get(self.current).unwrap_or(&'\0')
    }

    pub fn next_char(&self) -> char {
        *self.source.get(self.current + 1).unwrap_or(&'\0')
    }
}

impl Iterator for Scanner {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        let peek = self.peek();
        if !self.has_next() {
            // We reached end of input
            return None;
        }

        let mut next = Some(Token {
            name: Character { value: peek },
            position: self.current,
        });
        let next_token = next.as_mut().unwrap();

        match peek {
            '(' => {
                next_token.name = LeftParen;
            }
            ')' => {
                next_token.name = RightParen;
            }
            '|' => {
                next_token.name = Pipe;
            }
            '?' => {
                next_token.name = Mark;
            }
            '*' => {
                next_token.name = Star;
            }
            '+' => {
                next_token.name = Plus;
            }
            '.' => {
                next_token.name = Dot;
            }
            '\\' => {
                let next_char = self.next_char();
                let mut found_escaped_char = true;
                match next_char {
                    '\\' => {
                        next_token.name = EscapedSlash;
                    }
                    '(' => {
                        next_token.name = EscapedLeftParen;
                    }
                    ')' => {
                        next_token.name = EscapedRightParen;
                    }
                    '|' => {
                        next_token.name = EscapedPipe;
                    }
                    '?' => {
                        next_token.name = EscapedMark;
                    }
                    '*' => {
                        next_token.name = EscapedStar;
                    }
                    '+' => {
                        next_token.name = EscapedPlus;
                    }
                    '.' => {
                        next_token.name = EscapedDot;
                    }
                    _ => {
                        found_escaped_char = false;
                    }
                }
                if found_escaped_char && self.has_next() {
                    self.advance();
                }
            }
            _ => {
                // Any other character.
                // Nothing to be handled because by default
                // token name is TokenName::Character
            }
        }
        self.advance();
        next
    }
}
