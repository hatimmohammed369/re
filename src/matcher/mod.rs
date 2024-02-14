// Use a parsed regular expression to match against strings

use crate::parser::{parse, syntax_tree::*};

// Match operation outcome
#[allow(dead_code)]
#[derive(Debug)]
pub struct Match {
    // matched region of original target string
    slice: String,
    // It's the string beginning in index `begin`
    // up to but excluding index `end`
    // For instance:
    // if begin = end = 10, then `slice` is empty
    // if begin = 0 and end = 4, then `slice` is string of characters
    // target[0], target[1], target[2] and target[3]
    begin: usize, // begin index relative to original target string
    end: usize,   // end index (exlusive) relative to original target string
}

// Coordinator of the matching process
#[allow(dead_code)]
pub struct Matcher {
    // Parsed pattern
    pattern: Regexp,
    // String on which the search (pattern matching) is done
    target: Vec<char>,
    // Where last match ends
    current: usize,
    // True if this Matcher matched the trailing empty string
    // in its `target` string. False otherwise
    matched_trailing_empty_string: bool,
}

impl Matcher {
    // Create a new matcher from `pattern`
    // which is matched against `target`
    pub fn new(pattern: &str, target: &str) -> Result<Matcher, String> {
        let pattern = parse(pattern)?;
        let target = target.chars().collect();
        let current = 0;
        let matched_trailing_empty_string = false;
        Ok(Matcher {
            pattern,
            target,
            current,
            matched_trailing_empty_string,
        })
    }

    fn has_next(&self) -> bool {
        self.current < self.target.len()
    }

    fn advance(&mut self) {
        if self.has_next() {
            self.current += 1;
        }
    }

    fn unchecked_advance(&mut self) {
        self.current += 1;
    }

    // Assign a new target to match on
    pub fn assign_match_target(&mut self, target: &str) {
        self.target = target.chars().collect();
        self.current = 0;
        self.matched_trailing_empty_string = false;
    }

    // Find the next match (non-overlaping with previous match)
    pub fn find_match(&mut self) -> Option<Match> {
        match self.pattern.tag.clone() {
            ExpressionTag::EmptyExpression => self.empty_expression_match(),
            ExpressionTag::CharacterExpression { value, quantifier } => {
                self.character_expression_match(value, quantifier)
            }
            other => {
                eprintln!("Can not match parsed Regexp pattern with tag `{other:#?}`");
                panic!();
            }
        }
    }

    // Match current position in `target` against the empty regular expression
    // this function always succeeds, returning Some(Match)
    // because the empty string always matches even inside another empty string
    fn empty_expression_match(&mut self) -> Option<Match> {
        if self.current >= self.target.len() {
            if !self.matched_trailing_empty_string {
                // For completeness sake
                self.matched_trailing_empty_string = true;
                let begin = self.target.len();
                let end = begin;
                // use (self.target.len()-1) because Rust won't allow indexing with
                // (self.target.len())
                // But the matched slice is still the empty string
                let slice = String::new();
                Some(Match { slice, begin, end })
            } else {
                // Matched the trailing empty string
                // No more matches available
                None
            }
        } else {
            // Successfully matched an empty string
            let begin = self.current;
            let end = begin;
            let slice = String::new();
            // Make next search start further in `target`
            self.advance();
            Some(Match { slice, begin, end })
        }
    }

    // Match current position in `target` against a certain character
    // If there are still characters to match in `target`, match current against
    // the one given by the parsed expression `pattern`
    // Return None indicating failure
    #[allow(unused_variables)]
    fn character_expression_match(&mut self, value: char, quantifier: Quantifier) -> Option<Match> {
        while self.has_next() && self.target[self.current] != value {
            self.unchecked_advance();
        }

        if !self.has_next() {
            return self.empty_expression_match();
        }

        match quantifier {
            Quantifier::None | Quantifier::ZeroOrOne => {
                // Matching expressions `x` and `x?`

                let slice = String::from(value);
                let begin = self.current;
                // Make next search start further in `target`
                self.advance();
                let end = self.current;
                Some(Match { slice, begin, end })
            }
            low @ (Quantifier::ZeroOrMore | Quantifier::OneOrMore) => {
                // Matching expressions `x*` and `x+`

                let begin = self.current;
                let mut slice = String::with_capacity(self.target.len() - self.current);
                let mut matches_count = 0usize;
                while self.has_next() && self.target[self.current] == value {
                    self.unchecked_advance();
                    matches_count += 1;
                    slice.push(value);
                }

                if (matches!(low, Quantifier::ZeroOrMore) && matches_count > 0)
                    || (matches!(low, Quantifier::OneOrMore) && matches_count > 1)
                {
                    slice.shrink_to_fit();
                    let end = self.current;
                    Some(Match { slice, begin, end })
                } else {
                    self.empty_expression_match()
                }
            }
        }
    }
}
