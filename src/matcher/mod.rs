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
        let m = match self.pattern.tag.clone() {
            ExpressionTag::EmptyExpression => self.empty_expression_match(),
            ExpressionTag::CharacterExpression { value, quantifier } => {
                self.character_expression_match(value, quantifier)
            }
            ExpressionTag::DotExpression { quantifier } => self.dot_expression_match(quantifier),
            ExpressionTag::Group { quantifier } => self.group_match(quantifier),
            ExpressionTag::Alternation => self.alternation_match(),
            other => {
                eprintln!("FATAL ERROR:");
                eprintln!("Can not match parsed Regexp pattern with tag `{other:#?}`\n");
                panic!();
            }
        };
        if m.is_none() {
            // Attempt to match from the next character
            self.advance();
        }
        m
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

    // Match character `value` in current position
    // If Matcher reached end or current character is not `value` fail (return Option::None)
    fn character_expression_match(&mut self, value: char, quantifier: Quantifier) -> Option<Match> {
        if !self.has_next() || self.target[self.current] != value {
            // No more characters to match or current character is not `value`
            return match quantifier {
                // `x` and `x+` fail when they don't match at least one `x`
                Quantifier::None | Quantifier::OneOrMore => None,

                // `x?` and `x*` always match, either at least one `x` or the empty string ()
                _ => self.empty_expression_match(),
            };
        }

        match quantifier {
            Quantifier::None | Quantifier::ZeroOrOne => {
                // Matching expressions `x` and `x?`

                // Match a single x
                let slice = String::from(value);
                let begin = self.current;
                // Make next search start further in `target`
                self.advance();
                let end = self.current;
                Some(Match { slice, begin, end })
            }
            Quantifier::ZeroOrMore | Quantifier::OneOrMore => {
                // Matching expressions `x*` and `x+`

                // Match as many x's as possible
                let begin = self.current;
                let mut slice = String::with_capacity(self.target.len() - self.current + 1);
                while self.has_next() && self.target[self.current] == value {
                    self.unchecked_advance();
                    slice.push(value);
                }
                slice.shrink_to_fit();
                let end = self.current;

                Some(Match { slice, begin, end })
            }
        }
    }

    // Match current position in `target` against any character
    // Return None indicating failure
    fn dot_expression_match(&mut self, quantifier: Quantifier) -> Option<Match> {
        if !self.has_next() {
            match quantifier {
                // Matching either `.` or `.+` at end fails
                Quantifier::None | Quantifier::OneOrMore => Option::None,
                // Matching either `.?` or `.*` at end yields the empty string
                _ => self.empty_expression_match(),
            }
        } else {
            match quantifier {
                // There are more characters to match
                Quantifier::None | Quantifier::ZeroOrOne => {
                    // Matching expressions `.` and `.?`

                    // Consume one character
                    let slice = String::from(self.target[self.current]);
                    let begin = self.current;
                    // Make next search start further in `target`
                    self.advance();
                    let end = self.current;
                    Some(Match { slice, begin, end })
                }
                Quantifier::ZeroOrMore | Quantifier::OneOrMore => {
                    // Matching expressions `.*` and `.+`

                    // Consume all remaining characters
                    let begin = self.current;
                    self.current = self.target.len();
                    let end = self.current;
                    let slice = self.target[begin..].iter().collect::<String>();
                    Some(Match { slice, begin, end })
                }
            }
        }
    }

    fn group_match(&mut self, quantifier: Quantifier) -> Option<Match> {
        let parent = self.pattern.clone();
        let grouped_expression = self.pattern.children.borrow()[0].borrow().clone();
        self.pattern = grouped_expression;
        let child_match = match self.find_match() {
            Some(match_object) => match quantifier {
                // Grouped expression succeeded to match
                Quantifier::None | Quantifier::ZeroOrOne => Some(match_object),
                _ => {
                    let begin = match_object.begin;
                    let mut end = match_object.end;
                    let mut slice = match_object.slice;
                    slice.reserve(self.target.len() - self.current + 1);
                    while let Some(new_match) = self.find_match() {
                        if end == new_match.begin {
                            slice.push_str(&new_match.slice);
                            end = new_match.end;
                        } else {
                            self.current = end;
                            break;
                        }
                    }
                    slice.shrink_to_fit();
                    Some(Match { slice, begin, end })
                }
            },
            None => {
                // Grouped expression failed to match
                match quantifier {
                    // Matching (E) or (E)+ at end fails
                    Quantifier::None | Quantifier::OneOrMore => Option::None,

                    // Matching (E)? or (E)* at end yiels the empty string
                    _ => self.empty_expression_match(),
                }
            }
        };
        self.pattern = parent;
        child_match
    }

    fn alternation_match(&mut self) -> Option<Match> {
        let parent = self.pattern.clone();
        let mut child_match = None;
        let children = parent
            .children
            .borrow()
            .iter()
            .map(|rc| rc.borrow().clone())
            .collect::<Vec<_>>();
        for child in children {
            self.pattern = child;
            child_match = self.find_match();
            if child_match.is_none() {
                // Last match failed
                // Move backward one step because
                // find_match executed `self.current += 1`
                // after the last failed match
                if self.has_next() {
                    // Move back only when you have unprocessed characters
                    // because an expression like `(a+|a*)` in which `a*`
                    // will be matched at last will make Matcher hop indefinitely
                    // between self.target.len()-1 and self.target.len()
                    self.current -= 1;
                }
            } else {
                // One are of the branches matched
                // Return its match
                break;
            }
        }
        self.pattern = parent;
        child_match
    }
}
