#[allow(dead_code)]
pub mod tokens;

use tokens::*;

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

    pub fn peek(&self) -> char {
        *self.source.get(self.current).unwrap_or(&'\0')
    }
}

impl Iterator for Scanner {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        let peek = self.peek();
        if peek == '\0' {
            return None;
        }
        let next = Some(Token {
            name: TokenName::Character { value: peek },
            position: self.current,
        });
        self.current += 1;
        next
    }
}
