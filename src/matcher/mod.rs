// Use a parsed regular expression to match against strings

use crate::parser::{parse, syntax_tree::*};

// Match operation outcome
#[allow(dead_code)]
#[derive(Debug)]
pub struct Match {
    // Matched region of original target string
    // It's the string beginning in index `begin`
    // up to but excluding index `end`
    // For instance:
    // if begin = end = 10, then `slice` is empty
    // if begin = 0 and end = 4, then `slice` is string of characters
    // target[0], target[1], target[2] and target[3]
    slice: String,
    // Use a owned string so that Match objects
    // can be used independently of Matcher
    // and also Matcher internally stores its matching target
    // as a Vec<char> because String does NOT allow direct index
    // which Mathcer needs a lot
    begin: usize, // begin index relative to original target string
    end: usize,   // end index (exclusive) relative to original target string
}

#[allow(dead_code)]
// Backtracking information of a single expression
// Objects of this struct is used when an expression
// performs more than one backtrack
struct ExpressionBacktrackInfo {
    // The last item in this Vec represent the index of current pattern among its siblings in
    // current syntax tree level
    // All other items represent the index of its parents amongs their siblings with the same
    // syntax tree level
    index_sequence: Vec<usize>,

    // Position of first successful match of the associated expression
    first_match_begin: usize,

    // Upper exclusive bound next match MUST satisfy
    next_match_bound: usize,
    // First time the associated expression matches successfully
    // field `next_match_bound` is set match end index
    // Each time, including first, the associated expression successful matches,
    // field `next_match_bound` is, if positive, decremented by 1
    // Then next match of the associated expressoin must never exceed
    // index `next_match_bound`
}

// Coordinator of the matching process
#[allow(dead_code)]
pub struct Matcher {
    // Currently processed node of the given pattern syntax tree
    pattern: Regexp,
    // String on which the search (pattern matching) is done
    target: Vec<char>,
    // Where to start matching
    current: usize,
    // True if Matcher generated an empty string match in current position
    // False otherwise
    matched_empty_string: bool,

    // The last item in this Vec represent the index of current pattern among its siblings in
    // current syntax tree level
    // All other items represent the index of its parents amongs their siblings with the same
    // syntax tree level
    pattern_index_sequence: Vec<usize>,
    // Of course, root pattern (currently processed pattern) will have Vec
    // of one 0usize item, because root has no parent and its the zeroth child in its level

    // Backtrack info for currently processed pattern
    backtrack_info: Vec<ExpressionBacktrackInfo>,
}

impl Matcher {
    // Create a new matcher from `pattern`
    // which is matched against `target`
    pub fn new(pattern: &str, target: &str) -> Result<Matcher, String> {
        let pattern = parse(pattern)?;
        let target = target.chars().collect();
        let current = 0;
        let matched_empty_string = false;
        let pattern_index_sequence = vec![0];
        let backtrack_info = vec![];
        Ok(Matcher {
            pattern,
            target,
            current,
            matched_empty_string,
            pattern_index_sequence,
            backtrack_info,
        })
    }

    fn len(&self) -> usize {
        self.target.len()
    }

    fn has_next(&self) -> bool {
        self.current < self.len()
    }

    fn set_position(&mut self, pos: usize) {
        // Ensure that `self.current` is never set to value
        // greater than self.len() which is self.target.len()
        let pos = if pos > self.len() { self.len() } else { pos };

        let old_pos = self.current;
        self.current = pos;

        if old_pos < self.len() || !self.matched_empty_string {
            // !( old_pos == self.target.len() && self.matched_empty_string )
            // calling one of `self.set_position` or `self.advance`
            // ensures that old position (old_pos) is never greater than self.len() which is self.target.len()
            // so !(old_pos == self.target.len()) is never (old_pos > self.target.len())
            // hence it MUST be (old_pos < self.target.len())

            // It's NOT the case that we matched the trailing empty string
            // If we matched the trailing empty string and unset flag `matched_empty_string`
            // then Matcher would get stuck in a loop, indefinitely matching the trailing empty
            // because it setting and unsetting flag `matched_empty_string`
            self.matched_empty_string = false;
        }
    }

    fn ensure_position_bound(&mut self) {
        // If `self.current` has a value larger than self.len (equal to self.target.len())
        if !self.has_next() {
            // then set it to self.target.len()
            self.set_position(self.len())
        }
    }

    fn advance(&mut self) {
        // Move one step forward and
        // ensure self.current is never greater than self.target.len()
        self.current += 1;
        self.ensure_position_bound();
    }

    // Assign a new target to match on
    pub fn assign_match_target(&mut self, target: &str) {
        self.target = target.chars().collect();
        self.set_position(0);
        self.pattern_index_sequence.clear();
        self.backtrack_info.clear();
    }

    // Find the next match (non-overlapping with previous match)
    pub fn find_match(&mut self) -> Option<Match> {
        // WHY WE NEED A LOOP?
        // Because first match in target string may not be index 0
        // and hence we need to keep matching until we hit the first match
        // or reach end of target

        let mut match_attempt;
        loop {
            match_attempt = self.compute_match();
            if match_attempt.is_none() {
                // Last match failed
                if self.has_next() {
                    // Move forward to retry
                    // ADVANCE
                    self.advance();
                } else {
                    // No more characters to process
                    // HALT
                    break;
                }
            } else {
                // Return matched region
                if match_attempt.as_ref().unwrap().slice.is_empty() {
                    // Matched the empty string in current position
                    // Matcher MUST advance or it will loop endlessly
                    // matching the empty string at the same position
                    // because the empty string can match anywhere
                    self.advance();
                }
                break;
            }
        }
        match_attempt
    }

    fn compute_match(&mut self) -> Option<Match> {
        match self.pattern.tag.clone() {
            ExpressionTag::EmptyExpression => self.empty_expression_match(),
            ExpressionTag::CharacterExpression { value, quantifier } => {
                self.character_expression_match(value, quantifier)
            }
            ExpressionTag::DotExpression { quantifier } => self.dot_expression_match(quantifier),
            ExpressionTag::Group { quantifier } => self.group_match(quantifier),
            ExpressionTag::Alternation => self.alternation_match(),
            ExpressionTag::Concatenation => self.concatenation_match(),
        }
    }

    // Match current position in `target` against the empty regular expression
    // this function always succeeds, returning Some(Match)
    // because the empty string always matches even inside another empty string
    fn empty_expression_match(&mut self) -> Option<Match> {
        if !self.matched_empty_string || self.has_next() {
            // Not matched empty string here or not all characters processed
            // logical negation of: Matched trailing empty string
            // which is (self.matched_empty_string && !self.has_next())
            self.matched_empty_string = true;
            let slice = String::new();
            let begin = self.current;
            let end = self.current;
            Some(Match { slice, begin, end })
        } else {
            // Matched trailing empty string
            // Target string is completely consumed
            // NO MORE MATCHES FOR THIS TARGET
            None
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
                let mut slice = String::with_capacity(self.len() - self.current + 1);
                while self.has_next() && self.target[self.current] == value {
                    self.advance();
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
                    self.set_position(self.len());
                    let end = self.current;
                    let slice = self.target[begin..].iter().collect::<String>();
                    Some(Match { slice, begin, end })
                }
            }
        }
    }

    fn group_match(&mut self, quantifier: Quantifier) -> Option<Match> {
        let old_pattern = self.pattern.clone();
        let grouped_expression = self.pattern.children.borrow()[0].borrow().clone();
        self.pattern = grouped_expression;
        let child_match = match self.compute_match() {
            Some(match_object) => match quantifier {
                // Grouped expression succeeded to match
                // Matching (E) and (E)?
                Quantifier::None | Quantifier::ZeroOrOne => Some(match_object),

                // Matching (E)* and (E)+
                _ => {
                    let begin = match_object.begin;
                    let mut end = match_object.end;
                    let mut slice = match_object.slice;
                    slice.reserve(self.len() - self.current + 1);
                    while let Some(new_match) = self.compute_match() {
                        if end == new_match.begin {
                            slice.push_str(&new_match.slice);
                            end = new_match.end;
                        } else {
                            self.set_position(end);
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
                    // (E) and (E)+ fail when E fails
                    Quantifier::None | Quantifier::OneOrMore => Option::None,

                    // (E)? and (E)* match empty string when E fails
                    _ => self.empty_expression_match(),
                }
            }
        };
        self.pattern = old_pattern;
        child_match
    }

    fn alternation_match(&mut self) -> Option<Match> {
        let old_position = self.current;
        let old_pattern = self.pattern.clone();
        let mut child_match = None;
        let children = old_pattern
            .children
            .borrow()
            .iter()
            .map(|rc| rc.borrow().clone())
            .collect::<Vec<_>>();
        for child in children {
            self.pattern = child;
            child_match = self.compute_match();
            if child_match.is_none() {
                if self.has_next() {
                    self.set_position(old_position);
                }
            } else {
                // One are of the branches matched
                // Return its match
                break;
            }
        }
        self.pattern = old_pattern;
        child_match
    }

    fn concatenation_match(&mut self) -> Option<Match> {
        let old_position = self.current;

        let mut match_region = {
            let slice = String::with_capacity(self.len() - self.current + 1);
            let begin = self.current;
            let end = self.current;
            Match { slice, begin, end }
        };

        let old_pattern = self.pattern.clone();

        // Match first item
        self.pattern = old_pattern.children.borrow()[0].borrow().clone();
        match self.compute_match() {
            Some(match_obj) => {
                // First item matched
                match_region.slice.push_str(&match_obj.slice);
                match_region.end = match_obj.end;
            }
            None => {
                // First item FAILED
                // Restore old position and old pattern
                self.pattern = old_pattern.clone();
                self.set_position(old_position);
                return None;
            }
        };

        // Match remaining items
        let children = old_pattern.children.borrow()[1..]
            .iter()
            .map(|rc| rc.borrow().clone())
            .collect::<Vec<_>>();

        for child in children {
            self.pattern = child;
            match self.compute_match() {
                Some(match_obj) => {
                    // If this expression matched, then its match begins
                    // right after the match of its predecessor
                    // that's because Matcher field `self.current`
                    // is never incremented before doing the actual matching
                    // but it's incremented after a successful match
                    match_region.slice.push_str(&match_obj.slice);
                    match_region.end = match_obj.end;
                }
                None => {
                    // An item failed to match
                    // the whole concatenation expression fails
                    // Restore old position and old pattern
                    self.pattern = old_pattern;
                    self.set_position(old_position);
                    return None;
                }
            }
        }

        // Match successful
        match_region.slice.shrink_to_fit();
        // Restore old pattern
        self.pattern = old_pattern.clone();
        Some(match_region)
    }
}
