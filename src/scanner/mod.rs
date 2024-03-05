// Scanner module
// Split the pattern string into `Tokens` for the parser

#[allow(dead_code)]
pub mod tokens;

use tokens::{Token, TokenName::*};

use crate::report_fatal_error;

pub struct Scanner {
    // source string characters vector to allow fast access
    source: Vec<char>,
    // index of character in field (source) that's currenlty processed
    current: usize,
    // `found_empty_string` flag indicates whether we found the empty string token
    // in current position
    // when it's false it means we can attempt to generate Empty token
    // when it's true it means we already generated EmtpyString token or we could not do so
    // rather we should attempt to generate another token (if any remaining)
    found_empty_string: bool,
}

// an Iterator transforming source string into a tokens stream
// each toekn is generated on request
impl Scanner {
    pub fn new(source: &str) -> Scanner {
        // source characters as a vector for fast access
        let source = source.chars().collect::<Vec<_>>();
        // current (`processed` or `to be processed`) character
        let current = 0;
        // flag (found_empty_string) is false on start
        // because the empty string can occur anywhere with an abitrary string
        // even within the empty string (which is itself)
        let found_empty_string = false;
        Scanner {
            source,
            current,
            found_empty_string,
        }
    }

    // construct source string from field (self.source)
    pub fn get_source_string(&self) -> String {
        // pre-allocate at least `self.source.len()` bytes
        // to make appending characters faster
        self.source.iter().collect::<String>()
    }

    // get character at (index + offset) if this position exists
    // otherwise return \0
    fn get(&self, index: usize, offset: isize) -> char {
        let negative = offset < 0;
        // absolute value of `offset` as a usize integer
        let offset = offset.unsigned_abs();
        if negative {
            if index < offset {
                // index < |offset| ===> index - |offset| < 0
                // we return \0 to indicate the absence of characters
                // at this (negative) index (which is invalid)
                return '\0';
            }
            // index >= |offset| ===> index - |offset| >= 0
            // index - |offset| is valid index but still it may be out of bound
            // thus call Vec::get to get an Option which you can handle
            // if returned Option is None
            return *self.source.get(index - offset).unwrap_or(&'\0');
        }
        // index + offset a valid index but still it may be out of bound
        // again use Vec::get to avoid out-of-bound indexing
        *self.source.get(index + offset).unwrap_or(&'\0')
    }

    // advance the current character marker one step forward
    fn advance(&mut self) {
        if self.current < self.source.len() {
            self.current += 1;
        }
    }

    // check if we reached end of input
    // if current character marker index is a valid index
    // then we still have characters to process
    // otherwise we reached end of input
    fn has_next(&self) -> bool {
        self.current < self.source.len()
    }

    // get character right before currently processed character
    fn previous(&self) -> char {
        self.get(self.current, -1)
    }

    // get the currenlty processed character
    fn peek(&self) -> char {
        self.get(self.current, 0)
    }

    // get character right after currently processed character
    fn next_char(&self) -> char {
        self.get(self.current, 1)
    }
}

impl Iterator for Scanner {
    type Item = Token;

    // (Attempt to) generate a token for the current character
    // or an Empty token
    fn next(&mut self) -> Option<Token> {
        // First, try to generate an Empty token because
        // the empty string can appear anywhere within a string
        // even within the empty string (which is itself)
        let peek = self.peek();

        let prev = self.previous();
        // if certain characters "( | )" are adjacent with the former not escaped
        // we can generate an Empty token
        // self.source[self.current - 2] (if exists) is second to current character
        // it's where to look to check whether previous character is preceeded by a slash
        // which means it's escaped
        let is_prev_escaped = self.get(self.current, -2) == '\\';
        if !is_prev_escaped && !self.found_empty_string {
            // Set flag (self.found_empty_string) to not attempt to generate Empty token
            // if previous iteration did
            self.found_empty_string = true;

            // When does the scanner generate an `Empty` token?
            #[allow(unused_parens)]
            if (
                // CASE 1
                // "" (empty string)
                // source string is empty, emit `Empty` because it's the only token
                // which can appear in an emtpy string since it contains no characters at all
                self.source.is_empty() ||

                // CASE 2
                // "|..."
                // source string begins with |, emit `Empty` BEFORE the leading |
                (self.current == 0 && peek == '|') ||

                // CASE 3
                // "...|"
                // source string ends with |, emit `Empty` AFTER the trailing |
                (self.current == self.source.len() && prev == '|' && peek == '\0') ||

                // CASE 4
                // "...||..."
                // emit `Empty` AFTER | and BEFORE following |
                // in other words, emit `Empty` between two adjacent |'s
                (prev == '|' && peek == '|') ||

                // CASE 5
                // "...(|...)..."
                // emit `Empty` AFTER ( and BEFORE |
                (prev == '(' && peek == '|') ||

                // CASE 6
                // "...(...|)..."
                // emit `Empty` AFTER | and BEFORE )
                (prev == '|' && peek == ')') ||

                // CASE 7
                // "...()..."
                // emit `Empty` AFTER ( and BEFORE )
                (prev == '(' && peek == ')')
            ) {
                // Note that we do not call advance()
                // because Empty contains no characters at all
                // and hence we never actually moved
                // instead we set flag (found_empty_string) so
                // next time call `next` we do not visit this branch again
                return Some(Token {
                    name: Empty,
                    position: self.current,
                });
            }
            // we did not generate an Empty token at current position
            // but none of the three above cases occurred
            // we try to generate another token (if any remaining)
        }

        // Try to generate Empty token when calling `next` again
        self.found_empty_string = false;
        // note that even if flag (found_empty_string) was unset before calling Iterator::next
        // if execution reached to region of code then the return value of this call to
        // Iterator::next must return an Option::Some and then advancing
        // or Option::None which means we reached end of input
        // in both cases Scanner will NOT attempt to generate an Empty token
        // twice at the same position, we can't get stuck in a loop

        // When scanner is given an empty string as input
        // it generates an Empty token but self.current is still 0
        // when calling `next` again, it can NOT generate another Empty token
        // because flag `found_empty_string` is set by then
        // hence it reaches this region of code
        // the call `self.has_next()` performs the comparison
        // self.current(which is 0) < self.source.len() (also 0) which is 0 < 0
        // clearly false and then `return None` executes signaling the end of iterator
        if !self.has_next() {
            // We reached end of input and we can not generate
            // another token, not even Empty
            // All characters are consumed and we can not generate an Empty token
            // this iterator has no more elements, return None
            return None;
        }

        // By default assume the current character is an ordinary character
        // (not a metacharacter and not an escaped metacharacter)
        let mut next = Some(Token {
            name: Character { value: peek },
            position: self.current,
        });

        // a mutable (&mut) reference to Token object inside local variable `next`
        // we use this &mut reference to modify Token::name field in case current character
        // is not an ordinary character (metacharacter or an escaped metacharacter)
        let next_token = next.as_mut().unwrap(); //&mut Token

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
                if (self.current + 1) >= self.source.len() {
                    report_fatal_error("Unary operator slash \\ with no operand at end\n");
                }

                if let next_char @ ('\\' | '(' | ')' | '|' | '?' | '*' | '+' | '.') =
                    self.next_char()
                {
                    // Next character is a metacharacter

                    // Advanced one more time before the final advance at the end of function `next`
                    // to consume the metacharacter following the \ in `peek`

                    // Advance to consume the following (escaped by slash in `peek`) metacharacter
                    self.advance();

                    if let Character { value } = &mut next_token.name {
                        // Token data is the escaped metacharacter, not the slash in `peek`
                        *value = next_char;
                    }
                }
            }
            _ => {
                // Any other ordinary character.
                // that's, not a metacharacter and an escaped metacharacter
                // Nothing to be handled because by default
                // token name is TokenName::Character
            }
        }
        // move current character marker one step forward
        self.advance();
        next
    }
}
