// Use a parsed regular expression to match against strings

use crate::parser::{parse, syntax_tree::*};

// Match operation outcome
pub type Match = std::ops::Range<usize>;

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

    // If Some(K) then next performed match must never exceed index K
    backtrack_bound: Option<usize>,
}

impl Matcher {
    // Create a new matcher from `pattern`
    // which is matched against `target`
    pub fn new(pattern: &str, target: &str) -> Result<Matcher, String> {
        let pattern = parse(pattern)?;
        let target = target.chars().collect();
        let current = 0;
        let matched_empty_string = false;
        let pattern_index_sequence = vec![];
        let backtrack_info = vec![];
        let backtrack_bound = None;
        Ok(Matcher {
            pattern,
            target,
            current,
            matched_empty_string,
            pattern_index_sequence,
            backtrack_info,
            backtrack_bound,
        })
    }

    fn has_next(&self) -> bool {
        self.current < self.target.len()
    }

    fn set_position(&mut self, pos: usize) {
        let pos = if pos > self.target.len() {
            self.target.len()
        } else {
            pos
        };

        let old_pos = self.current;
        self.current = pos;

        if old_pos < self.target.len() || !self.matched_empty_string {
            // !( old_pos == self.target.len() && self.matched_empty_string )
            // calling one of `self.set_position` or `self.advance`
            // ensures that old position (old_pos) is never greater than self.target.len()
            // so !(old_pos == self.target.len()) is never (old_pos > self.target.len())
            // hence it MUST be (old_pos < self.target.len())

            // It's NOT the case that we matched the trailing empty string
            // If we matched the trailing empty string and unset flag `matched_empty_string`
            // then Matcher would get stuck in a loop, indefinitely matching the trailing empty
            // because it setting and unsetting flag `matched_empty_string`
            self.matched_empty_string = false;
        }
    }

    fn advance(&mut self) {
        if self.current < self.target.len() {
            self.current += 1;
        }
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
        // Track root expression
        self.dive();

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
                if match_attempt.as_ref().unwrap().is_empty() {
                    // Matched the empty string in current position
                    // Matcher MUST advance or it will loop endlessly
                    // matching the empty string at the same position
                    // because the empty string can match anywhere
                    self.advance();
                }
                break;
            }
        }

        // Abandon root expression
        self.bubble_up();

        match_attempt
    }

    fn supports_backtracking(expr: &Regexp) -> bool {
        match &expr.tag {
            ExpressionTag::Group { quantifier } => {
                // The group itself is quantified or the grouped expression
                // inside supports backtracking
                !matches!(quantifier, Quantifier::None)
                    || Self::supports_backtracking(&expr.children.borrow()[0].borrow())
            }
            ExpressionTag::CharacterExpression { quantifier, .. }
            | ExpressionTag::DotExpression { quantifier } => {
                // . or x are quantified
                !matches!(quantifier, Quantifier::None)
            }

            // Alternation and concatenation
            _ => {
                // At least one child supports backtracking
                expr.children
                    .borrow()
                    .iter()
                    .any(|child| Self::supports_backtracking(&child.borrow()))
            }
        }
    }

    fn compute_match(&mut self) -> Option<Match> {
        let computed_match = match self.pattern.tag.clone() {
            ExpressionTag::EmptyExpression => self.empty_expression_match(),
            ExpressionTag::CharacterExpression { value, quantifier } => {
                self.character_expression_match(value, quantifier)
            }
            ExpressionTag::DotExpression { quantifier } => self.dot_expression_match(quantifier),
            ExpressionTag::Group { quantifier } => self.group_match(quantifier),
            ExpressionTag::Alternation => self.alternation_match(),
            ExpressionTag::Concatenation => self.concatenation_match(),
        };

        // Destroy last used bound
        self.backtrack_bound = None;

        if computed_match.is_some()
            && Self::supports_backtracking(&self.pattern)
            // Root expression does not backtrack
            && self.pattern.parent.is_some()
        {
            // Record first match info for later use when backtracking

            let search_index = self.backtrack_info.binary_search_by(|info_entry| {
                info_entry.index_sequence.cmp(&self.pattern_index_sequence)
            });
            match search_index {
                Ok(item_index) => {
                    let expr_info = &mut self.backtrack_info[item_index];
                    let bound = &mut expr_info.next_match_bound;
                    // Decrement only if positive
                    *bound = bound.saturating_sub(1);
                }
                Err(insertion_index) => {
                    // This expression never matched before
                    // Insert a new info entry while maintaining order
                    let mut end = computed_match.as_ref().unwrap().end;
                    // Decrement only if positive
                    end = end.saturating_sub(1);
                    self.backtrack_info.insert(
                        insertion_index,
                        ExpressionBacktrackInfo {
                            index_sequence: self.pattern_index_sequence.clone(),
                            first_match_begin: computed_match.as_ref().unwrap().start,
                            next_match_bound: end,
                        },
                    )
                }
            }
        }

        computed_match
    }

    fn dive(&mut self) {
        // Begin matching a child of current patttern
        self.pattern_index_sequence.push(0);
    }

    fn bubble_up(&mut self) {
        // Done matching current pattern, go up back to its parent
        self.pattern_index_sequence.pop();
    }

    fn appoint_next_child(&mut self) {
        // Begin matching a sibling of current pattern
        *self.pattern_index_sequence.last_mut().unwrap() += 1;
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
            Some(Match {
                start: self.current,
                end: self.current,
            })
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
            // No more characters to match or current character is not `x`
            match quantifier {
                // Matching `x` or `x+` at end fails
                Quantifier::None | Quantifier::OneOrMore => None,

                // Expressions `x?` and `x*` at end match the empty string
                _ => self.empty_expression_match(),
            }
        } else {
            // There is at least one unmatched `x`
            match self.backtrack_bound {
                Some(bound) => {
                    match quantifier {
                        Quantifier::None => {
                            eprintln!("FATAL ERROR:");
                            eprintln!("Expression `{value}` was classified as backtracking!");
                            panic!();
                        }
                        Quantifier::OneOrMore if bound == self.current => {
                            // Expression `x+` can not match when starting and ending before current index
                            None
                        }
                        Quantifier::ZeroOrOne | Quantifier::ZeroOrMore if bound == self.current => {
                            // Expressions `x?` and `x*` match empty string before current index
                            self.empty_expression_match()
                        }
                        _ => {
                            // Match as many x's as possible but never exceed backtrack bound
                            let start = self.current;
                            while self.current <= bound && self.target[self.current] == value {
                                self.advance();
                            }
                            let end = self.current;

                            Some(Match { start, end })
                        }
                    }
                }
                None => {
                    match quantifier {
                        Quantifier::None | Quantifier::ZeroOrOne => {
                            // Matching expressions `x` and `x?`

                            // Remember, `x?` is greedy
                            // Match a single x
                            let start = self.current;
                            // Make next search start further in `target`
                            self.advance();
                            let end = self.current;
                            Some(Match { start, end })
                        }
                        Quantifier::ZeroOrMore | Quantifier::OneOrMore => {
                            // Matching expressions `x*` and `x+`

                            // Match as many x's as possible
                            let start = self.current;
                            while self.has_next() && self.target[self.current] == value {
                                self.advance();
                            }
                            let end = self.current;

                            Some(Match { start, end })
                        }
                    }
                }
            }
        }
    }

    // Match current position in `target` against any character
    // Return None indicating failure
    fn dot_expression_match(&mut self, quantifier: Quantifier) -> Option<Match> {
        if !self.has_next() {
            // No more characters to match
            match quantifier {
                // Matching `.` or `.+` at end fails
                Quantifier::None | Quantifier::OneOrMore => Option::None,

                // Expressions `.?` and `.*` at end match the empty string
                _ => self.empty_expression_match(),
            }
        } else {
            // There is at least one unmatched character
            match quantifier {
                Quantifier::None | Quantifier::ZeroOrOne => {
                    // Matching expressions `.` and `.?`

                    // Remember, `.?` is greedy
                    // Consume one character
                    let start = self.current;
                    // Make next search start further in `target`
                    self.advance();
                    let end = self.current;
                    Some(Match { start, end })
                }
                Quantifier::ZeroOrMore | Quantifier::OneOrMore => {
                    // Matching expressions `.*` and `.+`

                    // Consume all remaining characters
                    let start = self.current;
                    self.set_position(self.target.len());
                    let end = self.current;
                    Some(Match { start, end })
                }
            }
        }
    }

    fn group_match(&mut self, quantifier: Quantifier) -> Option<Match> {
        // Start tracking your children
        self.dive();

        let old_pattern = self.pattern.clone();
        self.pattern = old_pattern.children.borrow()[0].borrow().clone();

        let grouped_expression_mactch = {
            // Match a grouped expression

            match self.compute_match() {
                Some(mut match_object) => {
                    // Grouped expression match succeeded
                    match quantifier {
                        // Matching (E) and (E)?
                        Quantifier::None | Quantifier::ZeroOrOne => Some(match_object),
                        // Return whatever E returned

                        // Matching (E)* and (E)+
                        _ => {
                            // Consume as many E's as possible
                            while let Some(new_match) = self.compute_match() {
                                match_object.end = new_match.end;
                            }
                            Some(match_object)
                        }
                    }
                }
                None => {
                    // Grouped expression match failed
                    match quantifier {
                        // (E) and (E)+ fail when E fails
                        Quantifier::None | Quantifier::OneOrMore => Option::None,

                        // (E)? and (E)* match empty string when E fails
                        _ => self.empty_expression_match(),
                    }
                }
            }

            // Grouped expression computation ends
        };

        self.pattern = old_pattern;
        // Abandon your children
        self.bubble_up();

        grouped_expression_mactch
    }

    fn alternation_match(&mut self) -> Option<Match> {
        // Start tracking your children
        self.dive();

        let old_position = self.current;
        let old_pattern = self.pattern.clone();

        let alternation_match = {
            // Match an alternation expression

            let children = old_pattern
                .children
                .borrow()
                .iter()
                .map(|rc| rc.borrow().clone())
                .collect::<Vec<_>>();
            let mut child_match = None;
            for child in children {
                self.pattern = child;
                child_match = self.compute_match();
                if child_match.is_none() {
                    self.set_position(old_position);
                } else {
                    // One of the branches matched
                    // Return its match
                    break;
                }
                // Start tracking next child
                self.appoint_next_child();
            }

            // Alternation expression match computation ends
            child_match
        };

        self.pattern = old_pattern;
        // Abandon your children
        self.bubble_up();

        alternation_match
    }

    fn concatenation_match(&mut self) -> Option<Match> {
        // Start tracking your children
        self.dive();

        let old_position = self.current;
        let old_pattern = self.pattern.clone();

        let concatenation_match = {
            // Match a concatenation expression

            let mut backtracking_siblings_positions = Vec::<usize>::new();
            let children = old_pattern
                .children
                .borrow()
                .iter()
                .map(|rc| rc.borrow().clone())
                .collect::<Vec<_>>();
            let mut match_region_end = self.current;
            let mut child_index = 0usize;
            while child_index < children.len() {
                let child = &children[child_index];
                self.pattern = child.clone();

                match self.compute_match() {
                    Some(match_obj) => {
                        // If this expression matched, then its match begins
                        // right after the match of its predecessor
                        // that's because Matcher field `self.current`
                        // is never incremented before doing the actual matching
                        // but it's incremented after a successful match
                        match_region_end = match_obj.end;
                        if Self::supports_backtracking(&self.pattern)
                            && !backtracking_siblings_positions.contains(&child_index)
                        {
                            backtracking_siblings_positions.push(child_index);
                        }
                    }
                    None => {
                        match backtracking_siblings_positions.last().cloned() {
                            Some(sibling_index) => {
                                let (
                                    sibling_index,
                                    sibling_first_match_begin,
                                    sibling_next_match_bound,
                                ) = {
                                    let sibling_info = &self.backtrack_info[sibling_index];
                                    (
                                        *sibling_info.index_sequence.last().unwrap(),
                                        sibling_info.first_match_begin,
                                        sibling_info.next_match_bound,
                                    )
                                };

                                child_index = sibling_index;
                                self.set_position(sibling_first_match_begin);
                                self.backtrack_bound = Some(sibling_next_match_bound);
                                *self.pattern_index_sequence.last_mut().unwrap() = child_index;
                                continue;
                            }
                            None => {
                                // An item failed to match and none of its
                                // preceeding siblings can backtrack
                                // The whole concatenation expression fails
                                // Restore old position and old pattern
                                self.pattern = old_pattern;
                                self.set_position(old_position);
                                // Abandon your children
                                self.bubble_up();
                                return None;
                            }
                        }
                    }
                }

                child_index += 1;
                // Start tracking next child
                self.appoint_next_child();
            }

            // Concatenation expression match computation ends
            Some(Match {
                start: old_position,
                end: match_region_end,
            })
        };

        self.pattern = old_pattern.clone();
        // Abandon your children
        self.bubble_up();

        concatenation_match
    }
}
