#[allow(dead_code)]
pub mod tokens;

use tokens::{Token, TokenName::*};

pub struct Scanner {
    source: Vec<char>,
    current: usize,
    found_empty_string: bool,
}

impl Scanner {
    pub fn new(source: &str) -> Scanner {
        let source = source.chars().collect::<Vec<_>>();
        let current = 0;
        let found_empty_string = false;
        Scanner {
            source,
            current,
            found_empty_string,
        }
    }

    pub fn get(&self, index: usize, offset: isize) -> char {
        let negative = offset < 0;
        let offset = offset.unsigned_abs();
        if negative {
            if index < offset {
                return '\0'
            }
            return *self.source.get(index - offset).unwrap_or(&'\0')
        }
        *self.source.get(index + offset).unwrap_or(&'\0')
    }

    pub fn advance(&mut self) {
        self.current += 1;
    }

    pub fn has_next(&self) -> bool {
        self.current < self.source.len()
    }

    pub fn previous(&self) -> char {
        self.get(self.current, -1)
    }

    pub fn peek(&self) -> char {
        self.get(self.current, 0)
    }

    pub fn next_char(&self) -> char {
        self.get(self.current, 1)
    }
}

impl Iterator for Scanner {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        // First, try to generate an EmptyString token because
        // the empty string itself can appear anywhere within a string
        // even within the empty string (which is itself)
        let peek = self.peek();

        let prev = self.previous();
        // self.source[self.current - 2] (if exists) is second to current character
        // it's where to look if we want to know whether previous character is preceeded by a slash
        // which means it's escaped
        let is_prev_escaped = self.get(self.current, -2) == '\\';
        if !is_prev_escaped && !self.found_empty_string {
            // Set flag (self.found_empty_string) to not attempt to generate EmptyString token
            // if previous iteration did
            self.found_empty_string = true;
            // There are 3 cases in which there is an empty string between metacharacters
            // CASE 1: An empty source string or a string starting with |
            if (prev == '\0' && (peek == '\0' || peek == '|'))
                // CASE 2: ( followed by either | or )
                || (prev == '(' && (peek == '|' || peek == ')'))
                // CASE 3: | followed by nothing or ) or another |
                || (prev == '|' && (peek == '\0' || peek == '|' || peek == ')'))
            {
                // note that we do not call advance()
                // because EmptyString contains no characters at all
                // and hence we never actually moved
                // instead we set flag (found_empty_string) so
                // next time call `next` we do not visit this branch again
                return Some(Token {
                    name: EmptyString,
                    position: self.current,
                });
            }
        }

        // Try to generate EmptyString token when calling `next` again
        self.found_empty_string = false;

        // When scanner is given an empty string as input
        // it generates an EmptyString token but self.current is still 0
        // next time calling `next` it can not generate another EmptyString token
        // hence it reaches this region of code
        // the call `self.has_next()` performs the comparison
        // self.current(which is 0) < self.source.len() (also 0) which is 0 < 0
        // clearly false and then `return None` executes signaling the end of iterator
        if !self.has_next() {
            // We reached end of input and we can not generate
            // another token, not even EmptyString
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
