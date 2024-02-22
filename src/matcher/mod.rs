// Use a parsed regular expression to match against strings

use crate::parser::{parse, syntax_tree::*};

// Match operation outcome
pub type Match = std::ops::Range<usize>;

#[allow(dead_code)]
// If an expression E can backtrack (like a+)
// then each time it successfully matches a range
// record that range such that if it needs to backtrack
// Matcher can use its last record range to force it
// to match a smaller range
struct ExpressionBacktrackInfo {
    // The last item in this Vec represent the index of current pattern among its siblings
    // within its level in parsed pattern syntax tree
    // All other items represent the index of its parents among their siblings within
    // the their respective parsed pattern syntax tree level
    index_sequence: Vec<usize>,

    // Position of first successful match of the associated expression
    // This field is never mutated once set
    first_match_begin: usize,

    // Upper exclusive bound next match MUST satisfy
    next_match_bound: usize,
    // First time the associated expression matches successfully
    // field `next_match_bound` is set to that successful match end index
    // Each time, including first, the associated expression successful matches,
    // field `next_match_bound` is, if positive, decremented by 1
    // Then next match of the associated expressoin must never exceed
    // index `next_match_bound`

    // Did the associated expression backtrack all the
    // way back to index 0?
    backtracked_to_index_0: bool,
    // If this flag is set, then the associated expression
    // can no longer backtrack or match
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
    // For instance, a value of X = vec![0, 3, 4] means that currently processed pattern (subexpression)
    // is the fourth (X[2]) child within its level
    // its parent is the third(X[1]) child within the level above
    // its grandparent is the root (X[0])

    // Backtrack info of all subexpressions which can backtrack
    backtrack_table: Vec<ExpressionBacktrackInfo>,

    // Index of active backtrack info in `backtrack_table`
    backtrack_active_entry_index: Option<usize>,
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
        let backtrack_table = vec![];
        let backtrack_active_entry_index = None;
        Ok(Matcher {
            pattern,
            target,
            current,
            matched_empty_string,
            pattern_index_sequence,
            backtrack_table,
            backtrack_active_entry_index,
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
        self.backtrack_table.clear();
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
        // An arbitrary expression E supports backtracking if:
        // 1 - It's quantified, in other words it's preceeding a quantifier (like .*)
        // 2 - At least one of its children supports backtracking (like (a+|c) because a+ can backtrack)

        match &expr.tag {
            // The empty expression can match anywhere
            // It doesn't need backtracking
            ExpressionTag::EmptyExpression => false,

            ExpressionTag::CharacterExpression { quantifier, .. }
            | ExpressionTag::DotExpression { quantifier } => {
                // . or x are quantified
                !matches!(quantifier, Quantifier::None)
            }

            ExpressionTag::Group { quantifier } => {
                // The group itself is quantified or the grouped expression
                // inside supports backtracking
                !matches!(quantifier, Quantifier::None)
                    || Self::supports_backtracking(&expr.children.borrow()[0].borrow())
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

    // ALL EXPRESSIONS MUST RESTORE OLD POSITION WHEN FAILING TO MATCH
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
        // BUT WHY?
        // If last match is successful, then it makes no sense to make next expression
        // backtrack making current one backtrack again (loop)

        // If last match failed, then we search for previous siblings of current expression
        // which can backtrack and hence we need a new backtrack bound
        self.backtrack_active_entry_index = None;

        // If current expression successfully matched AND
        // It can backtrack (like .?) AND
        // It's not root expression (it makes no sense to have root expression request a backtrack, it has no siblings)
        if computed_match.is_some()
            && Self::supports_backtracking(&self.pattern)
            // Root expression does not backtrack
            && self.pattern.parent.is_some()
        {
            // THEN
            // Record first match info for later use when backtracking

            // Attempt to find current expression info entry
            let search_index = self.backtrack_table.binary_search_by(|info_entry| {
                info_entry.index_sequence.cmp(&self.pattern_index_sequence)
            });
            match search_index {
                Ok(item_index) => {
                    // Found entry
                    // Decrement `next_match_bound` to force it to
                    // match a smaller range next time (when it backtracks)

                    let expr_info = &mut self.backtrack_table[item_index];
                    let bound = &mut expr_info.next_match_bound;
                    // To stop the associated expression from backtracking endlessly
                    expr_info.backtracked_to_index_0 = *bound == 0;
                    // Decrement only if positive
                    *bound = bound.saturating_sub(1);
                }
                Err(insertion_index) => {
                    // This expression never matched before
                    // Insert a new info entry while maintaining order of all entries
                    // Entries (ExpressionBacktrackInfo objects) are sorted by field 'index_sequence'

                    let mut end = computed_match.as_ref().unwrap().end;
                    // Decrement only if positive
                    end = end.saturating_sub(1);
                    self.backtrack_table.insert(
                        insertion_index,
                        ExpressionBacktrackInfo {
                            index_sequence: self.pattern_index_sequence.clone(),
                            first_match_begin: computed_match.as_ref().unwrap().start,
                            next_match_bound: end,
                            backtracked_to_index_0: false,
                            // No need to `backtracked_to_index_0` to expression `end == 0`
                            // because it's means this expression just matched is an empty
                            // expression (does not backtrack, and hence condition `Self::supports_backtracking` is false)
                            // Or, this expression quantified by ? or * which are greedy be default
                            // but they matched a string ending in index 0 (thus start is also 0 because start >= 0)
                            // meaning the target string is an empty string (no backtrack needed)
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

    // EMPTY EXPRESSIONS:
    // "" `an empty pattern string`
    // ()
    // ...(|...)... `between ( and |`
    // ...(...|)... `between | and )`
    // |... `before the leading |`
    // ...| `after the trailing |`
    // ...||... `between the two |`

    // HOW TO MATCH AN EMPTY STRING EXPRESSION:
    // Match current position in `target` against the empty regular expression
    // this function always succeeds, returning Some(Match)
    // because the empty string always matches even inside another empty string
    // There is only one case when this function fails (return None)
    // it's when Matcher matched the trailing empty string (empty string after the last valid index)
    // it makes sense to stop there or Matcher would endlessly match that trailing empty string
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

    // CHARACTER EXPRESSIONS:
    // x \ x? \ x* \ x+
    // x is a single character
    // Also, x is not a metacharacter or it's an escaped metacharacter
    // metacharacters are defined in file `grammar`
    // for instance, k+ is a character expression

    // HOW TO MATCH A CHARACTER EXPRESSION:
    // Consume the longest (bounded above, if Matcher is backtracking) sequence of contiguous characters `value`
    // When target is consumed, return None indicating failure
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
            match self.backtrack_active_entry_index {
                Some(index) => {
                    // Current match is bounded (a backtrack match)
                    // It starts from self.current (ExpressionBacktrackInfo field `first_match_begin`)
                    // Current match MUST never exceed a certain index
                    let active_entry = &self.backtrack_table[index];
                    let bound = active_entry.next_match_bound;
                    if !active_entry.backtracked_to_index_0 {
                        match quantifier {
                            Quantifier::None => {
                                // Expression `x` was mistakenly classified as a backtracking expression
                                // Or, last used backtrack_active_entry_index was not destroyed
                                eprintln!("FATAL ERROR:");
                                eprintln!("Expression `{value}` was classified as backtracking!");
                                panic!();
                            }
                            Quantifier::OneOrMore if bound == self.current => {
                                // Expression `x+` can not generate match starting and ending
                                // at the same index (name self.current)
                                // For instance:
                                // If `x+` matched exactly one `x` then its backtrack info is range 4..5
                                // its ExpressionBacktrackInfo has value 4 in field 'first_match_begin'
                                // and value 4 in field `next_match_bound`
                                // (it's 5-1, decremented after the match in function compute_match)
                                // Thus next time it must make a match starting and ending at index 4
                                // Which is the empty string, but `x+` must match at least one `x`
                                // And now it can only match the empty string, IT FAILED
                                None
                            }
                            Quantifier::ZeroOrOne | Quantifier::ZeroOrMore
                                if bound == self.current =>
                            {
                                // Expressions `x?` and `x*` match empty string between
                                // self.current (first_match_begin) and self.current (next_match_bound)
                                self.empty_expression_match()
                            }
                            _ => {
                                // Match as many x's as possible but never exceed backtrack bound (next_match_bound)
                                let start = self.current;
                                while self.current <= bound && self.target[self.current] == value {
                                    self.advance();
                                }
                                let end = self.current;

                                Some(Match { start, end })
                            }
                        }
                    } else {
                        // This expression expired its chances
                        // No more backtracking or matching
                        None
                    }
                }
                None => {
                    // Current match is NOT bounded
                    // Consume as much characters as possible
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

                            // Remember, `x*` and `x+` are greedy
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

    // DOT EXPRESSIONS:
    // . \ .? \ .* \ .+
    // Use a literal dot

    // HOW TO MATCH A DOT EXPRESSION:
    // Consume the longest (bounded above, if Matcher is backtracking) sequence of characters
    // When target is consumed, return None indicating failure
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

                    // Remember, `.*` and `.+` are greedy
                    // Consume all remaining characters
                    let start = self.current;
                    self.set_position(self.target.len());
                    let end = self.current;
                    Some(Match { start, end })
                }
            }
        }
    }

    // GROUP/GROUPED EXPRESSIONS:
    // (E) where E is also an expression
    // for instance, (a+|b) is group/grouped expression

    // HOW TO MATCH GROUPED EXPRESSION:
    // Match whatever grouped expression matched
    // and then apply the quantifiers after the group itself
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
                            // Make expression E match as much as possible
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

        // Restore parent pattern to process remaining siblings of current pattern
        self.pattern = old_pattern;
        // Abandon your children
        self.bubble_up();

        grouped_expression_mactch
    }

    // ALTERNATION EXPRESSIONS:
    // (E1|E2|...|E_n) where E1,E2,...,E_n are also expressions
    // for instance, a|b.c|x is an alternation expression

    // HOW TO MATCH AN ALTERNATION EXPRESSION:
    // Match children in order from first to last
    // return the match of the first matching child
    fn alternation_match(&mut self) -> Option<Match> {
        // Start tracking your children
        self.dive();

        let old_position = self.current;
        let old_pattern = self.pattern.clone();

        let alternation_match = {
            // Match an alternation expression

            let children = self
                .pattern
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
                    // Return to original position this alternation expression started at
                    // to make all its children start matching from the same position
                    self.set_position(old_position);
                    // If last child failed to match, the above call
                    // automatically restores old position where this alternation started matching
                } else {
                    // One of the branches matched
                    // The whole alternation expression has matched
                    // Return its match
                    break;
                }
                // Start tracking next child
                self.appoint_next_child();
            }

            // Alternation expression match computation ends
            child_match
        };

        // Restore parent pattern to process remaining siblings of current pattern
        self.pattern = old_pattern;
        // Abandon your children
        self.bubble_up();

        alternation_match
    }

    // CONCATENATION EXPRESSIONS:
    // E1E2...E_n, where E1, E2, ..., E_n are also expressions
    // for instance, a.(a+|b*)c* is a concatenation expression with
    // E1 = a, E2 = ., E3 = (a+|b*), E4 = c*

    // HOW TO MATCH A CONCATENATION EXPRESSION:
    // If at least one child fails, then the whole expression fails too
    // Otherwise return the range starting from first child match
    // and ending with last child match
    fn concatenation_match(&mut self) -> Option<Match> {
        // Start tracking your children
        self.dive();

        let old_position = self.current;
        let old_pattern = self.pattern.clone();

        let concatenation_match = {
            // Match a concatenation expression

            let children = self
                .pattern
                .children
                .borrow()
                .iter()
                .map(|rc| rc.borrow().clone())
                .collect::<Vec<_>>();
            // Positions (indices) of children supporting backtrack in Vec `children`
            // and in Matcher field `backtrack_table`
            let mut backtracking_siblings_positions = Vec::<(usize, usize)>::new();
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
                        if Self::supports_backtracking(&self.pattern) {
                            if !backtracking_siblings_positions // do not list the same sibling twice
                                .iter()
                                .any(|(child_idx, _)| *child_idx == child_index)
                            {
                                backtracking_siblings_positions
                                    .push((child_index, self.backtrack_table.len() - 1));
                            }
                        }
                    }
                    None => {
                        let nearest_preceeding_backtracking_sibling =
                            backtracking_siblings_positions
                                .iter()
                                .filter(|(sibling_index, table_entry_index)| {
                                    // Sibling is actually before current index (child_index)
                                    // and it has NOT backtracked to index 0
                                    *sibling_index < child_index
                                        && !self.backtrack_table[*table_entry_index]
                                            .backtracked_to_index_0
                                })
                                .next_back();
                        match nearest_preceeding_backtracking_sibling.cloned() {
                            Some((sibling_index, table_entry_index)) => {
                                // Start matching from that backtracking preceeding sibling
                                child_index = sibling_index;
                                let table_entry = &self.backtrack_table[table_entry_index];
                                // Restore position when this concatenation began matching
                                self.set_position(table_entry.first_match_begin);
                                // Point to backtrack entry in table
                                self.backtrack_active_entry_index = Some(table_entry_index);
                                // Fix subexpressions index tracker
                                *self.pattern_index_sequence.last_mut().unwrap() = child_index;
                                continue;
                            }
                            None => {
                                // An item failed to match and none of its
                                // preceeding siblings can backtrack
                                // The whole concatenation expression fails
                                // Restore parent pattern to process remaining siblings of current pattern
                                self.pattern = old_pattern;
                                // Restore old position
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
