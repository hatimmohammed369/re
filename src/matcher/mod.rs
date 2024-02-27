// Use a parsed regular expression to match against strings

use crate::parser::{parse, syntax_tree::*};

const METACHARACTERS: [char; 7] = ['(', ')', '\\', '|', '*', '.', '?'];

pub fn escape(pattern: &str) -> String {
    // Escape all metacharacters in `pattern`
    let mut escaped = String::with_capacity(
        // Possible each character is a metacharacter
        // requiring two slashes
        3 * pattern.len(),
    );
    for ch in pattern.chars() {
        if METACHARACTERS.contains(&ch) {
            // Add a slash to escaped the metacharacter
            // You need to write one slash BUT Rust needs you to escape this one slash
            // so actually we need 2 slashes
            escaped.push('\\'); // Rust escaping slash
            escaped.push('\\'); // Regexp escaping slash
        }
        escaped.push(ch);
    }
    escaped.shrink_to_fit();
    escaped
}

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

    // Position of last successful match of the associated expression
    last_match_start: usize,

    // End position of last successful match of the associated expression
    last_match_end: usize,

    // True if the associated expression backtracked until it attempted to
    // match with the range starting and ending at `last_match_start`
    // False otherwise
    backtracked_to_last_match_start: bool,
    // If true, reset fields of this ExpressionBacktrackInfo unless its
    // associated expression has NO preceeding backtrackable sibling
    // with its respective field `backtracked_to_last_match_start` set false
    // If it has no such sibling then its parent (a concatenation) fails to match
}

// Coordinator of the matching process
pub struct Matcher {
    // Currently processed node of the given pattern syntax tree
    pattern: Regexp,

    // String on which the search (pattern matching) is done
    target: Vec<char>,
    // Direct indexing, not supported by String, is usually needed
    // so it's better to use a Vec rather than a String

    // Current position in target string (Vec field `target`)
    pos: usize,

    // True if Matcher generated match for the trailing empty string
    // if field `pattern` can match the empty string
    // False otherwise
    // It's the guard keeping Matcher from infinitely matching the trailing empty string
    // because if expression E can match the empty string
    // then it can do so arbitrarily
    matched_trailing_empty_string: bool,

    // The last item in this Vec represent the index of current pattern among its siblings in
    // current syntax tree level
    // All other items represent the index of its parents amongs their siblings with the same
    // syntax tree level
    pattern_index_sequence: Vec<usize>,
    // Of course, root pattern (Regexp parsed in Matcher::new) will have Vec
    // of one 0usize item, because root has no parent and its the zeroth (first) child in its level
    // For instance, a value of X = vec![0, 3, 4] means that currently processed pattern (subexpression)
    // is the fourth (X[2]) child within its level
    // its parent is the third(X[1]) child within the level above
    // its grandparent is the root (X[0])

    // Backtrack info of all subexpressions which can backtrack
    backtrack_table: Vec<ExpressionBacktrackInfo>,
}

impl Matcher {
    // Create a new matcher from `pattern`
    // which is matched against `target`
    pub fn new(pattern: &str, target: &str) -> Result<Matcher, String> {
        let pattern = parse(pattern)?;
        let target = target.chars().collect();
        let pos = 0;
        let matched_trailing_empty_string = false;
        let pattern_index_sequence = vec![];
        let backtrack_table = vec![];
        Ok(Matcher {
            pattern,
            target,
            pos,
            matched_trailing_empty_string,
            pattern_index_sequence,
            backtrack_table,
        })
    }

    // Current "normalized" position
    // Always return something less than or equal to target length
    #[inline(always)]
    fn current(&self) -> usize {
        std::cmp::min(self.pos, self.target.len())
    }

    #[inline(always)]
    fn has_next(&self) -> bool {
        self.pos < self.target.len()
    }

    #[inline(always)]
    fn set_position(&mut self, pos: usize) {
        self.pos = pos;
    }

    #[inline(always)]
    fn advance(&mut self) {
        self.pos += 1;
    }

    // Assign a new target to match on
    pub fn assign_match_target(&mut self, target: &str) {
        self.target = target.chars().collect();
        self.set_position(0);
        // Stop tracking expressions
        self.pattern_index_sequence.clear();
        // Do not use old backtrack info
        self.backtrack_table.clear();
    }

    fn supports_backtracking(expr: &Regexp) -> bool {
        // An arbitrary expression E supports backtracking if:
        // 1 - It's quantified, in other words it's succeeded by a quantifier, like `.*`
        // 2 - At least one of its children supports backtracking, like `(a+|c)` because a+ can backtrack

        match &expr.tag {
            // The empty expression can match anywhere
            // It doesn't need backtracking
            ExpressionTag::EmptyExpression => false,

            ExpressionTag::CharacterExpression { quantifier, .. } => {
                // . or x are quantified

                // It's not the case that this expression has no quantifier
                // in other words, it's quantified with one of ? \ * \ +
                !matches!(quantifier, Quantifier::None)
                // Variant Quantifier::None represent the idea of `no quantifier`
            }

            ExpressionTag::Group { quantifier } => {
                // The group itself is quantified or the grouped expression
                // inside supports backtracking

                // It's not the case that this expression has no quantifier
                // in other words, it's quantified with one of ? \ * \ +
                !matches!(quantifier, Quantifier::None)
                    || Self::supports_backtracking(&expr.children.borrow()[0].borrow())
                // Variant Quantifier::None represent the idea of `no quantifier`
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
        let computed_match = match self.pattern.tag {
            ExpressionTag::EmptyExpression => self.empty_expression_match(),

            ExpressionTag::CharacterExpression { value, quantifier } => {
                self.character_expression_match(value, quantifier)
            }

            ExpressionTag::Group { quantifier } => self.group_match(quantifier),

            ExpressionTag::Alternation => self.alternation_match(),
            ExpressionTag::Concatenation => self.concatenation_match(),
        };

        // If current expression successfully matched AND
        // It can backtrack (like .?) AND
        // It's not root expression (it makes no sense to have root expression request a backtrack, it has no siblings)
        if computed_match.is_some()
            && Self::supports_backtracking(&self.pattern)
            // Root expression does not backtrack
            && self.pattern.parent.is_some()
        {
            // Record first match info for later use when backtracking

            let (start, end) = {
                let temp = computed_match.as_ref().unwrap();
                (temp.start, temp.end)
            };

            // Attempt to find current expression info entry
            let search_index = self.backtrack_table.binary_search_by(|info_entry| {
                info_entry.index_sequence.cmp(&self.pattern_index_sequence)
            });
            match search_index {
                Ok(item_index) => {
                    // Found entry
                    let expr_info = &mut self.backtrack_table[item_index];
                    // Reset `last_match_start` to make the associated expression of this entry usable
                    expr_info.last_match_start = start;
                    // Update other values
                    expr_info.last_match_end = end;
                    // When matching, expression `last_match_end - 1` is used as current bound match
                    // so if the expression made a match, variable `end` will have smaller value
                    // than field `last_match_end` because end it's at most (last_match_end - 1)
                    expr_info.backtracked_to_last_match_start = start == end;
                }
                Err(insertion_index) => {
                    // This expression never matched before
                    // Insert a new info entry while maintaining order of all entries
                    // Insert at index found by binary search stored in `search_index`
                    // Entries (ExpressionBacktrackInfo objects) are sorted by field 'index_sequence'

                    self.backtrack_table.insert(
                        insertion_index,
                        ExpressionBacktrackInfo {
                            index_sequence: self.pattern_index_sequence.clone(),
                            last_match_start: start,
                            last_match_end: end,
                            backtracked_to_last_match_start: start == end,
                        },
                    )
                }
            }
        }

        computed_match
    }

    #[inline(always)]
    fn dive(&mut self) {
        // Begin matching a child of current patttern
        self.pattern_index_sequence.push(0);
    }

    #[inline(always)]
    fn bubble_up(&mut self) {
        // Done matching current pattern, go up back to its parent
        self.pattern_index_sequence.pop();
    }

    #[inline(always)]
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

    // Always match
    #[inline(always)]
    fn empty_expression_match(&mut self) -> Option<Match> {
        // BUT ensure Matcher advances (call self.advance) later
        // otherwise Matcher would loop endlessly matching the empty string
        // at the same position because empty expression match NEVER fails
        let current = self.current();
        Some(Match {
            start: current,
            end: current,
        })
    }

    // CHARACTER & DOT EXPRESSIONS:
    // x \ x? \ x* \ x+
    // . \ .? \ .* \ .+
    // x is a single character
    // Also, x is not a metacharacter or it's an escaped metacharacter
    // metacharacters are defined in file `grammar`
    // for instance, k+ is a character expression

    // HOW TO MATCH CHARACTER & DOT EXPRESSIONS?
    // If field `value`, found in field `tag` of this expression, is Option::<char>::None
    // then this character expression is actually a dot expression
    // If this expression is `.`, consume a single character
    // If this expression `x`, consume a single `x` only if current character is `x`

    // For any other backtracking character (or dot) expression (a . or an arbitrary character followed by one of ? \ * \ +)
    // if the expression NEVER matched before
    // allow it to consume as many x's (or characters) as possible
    // If the expression DID match before, use field `last_match_end` in its associated
    // ExpressionBacktrackInfo in `self.backtrack_table` to force it to match a smaller range
    // Temporarily subtract one (if possible) from field `last_match_end` and use it
    // as a bound of current match

    // Return Option::<std::ops::Range>::Some(...) on success
    // Return Option::<std::ops::Range>::None on failure
    fn character_expression_match(
        &mut self,
        value: Option<char>,
        quantifier: Quantifier,
    ) -> Option<Match> {
        // Choose match bound (where the match ends, exclusive of course)
        // A non-backtracking expression (`.` or `x`) gets bound (self.current + 1)
        // For any other expression subtract one (if possible, if not use 0) from field `last_match_end`
        // found in its associated entry (ExpressionBacktrackInfo object) in self.backtrack_table
        let match_bound = {
            // Find backtrack table entry of this expression
            // Expressions `.` and `x` DO NOT have such entries, NEVER
            // If they do, it MUST be a bug in function Self::supports_backtracking
            let table_entry = self.backtrack_table.iter().find(|entry| {
                // Grab entry whose index sequence matches that of currently processed pattern
                entry.index_sequence == self.pattern_index_sequence
            });
            match table_entry {
                // This expression can backtrack (a quantified `.` or `x`, for instance `.+`)
                // AND also it has a backtrack table entry
                // Use its `last_match_end` to force it to match a smaller range
                Some(info) => info.last_match_end.saturating_sub(1),

                None => {
                    // This expression NEVER matched before
                    // OR it does not support backtracking (like `x` or `.`)
                    match quantifier {
                        // Expressions `.` and `x`
                        // Consume exactly one character
                        Quantifier::None | Quantifier::ZeroOrOne => self.pos + 1,

                        // Quantified `.` or `x`
                        // Consume as many characters as possible
                        _ => self.target.len(),
                    }
                }
            }
        };

        let start = self.current();
        // Consume characters as long as there are unmatched characters
        // only if this expression is a dot expression or the next unmatched
        // character is `x` and this expression is one of: x \ x? \ x* \ x+
        while self.has_next() // Ensure self.pos is in bounds of Vec self.target
            && self.pos < match_bound // Ensure match NEVER exceeds its bound
            // Consume characters anyway or stop if the given character is NOT in current position
            && !(value.is_some() && self.target[self.pos] != value.unwrap())
        {
            self.advance();
        }
        let end = self.current();

        if start == end {
            // Empty range
            match quantifier {
                // Expressions . \ .+ \ x \ x+ MUST match at least one character
                // They fail if they `matched` an empty range
                Quantifier::None | Quantifier::OneOrMore => Option::<Match>::None,

                // Expressions .? \ .* \ x? \ x* match the empty string
                // when they fail to match at least one character
                _ => self.empty_expression_match(),
            }
        } else {
            Some(Match { start, end })
        }
    }

    // GROUP/GROUPED EXPRESSIONS:
    // (E) where E is also an expression
    // for instance, (a+|b) is group/grouped expression

    // HOW TO MATCH GROUPED EXPRESSION:
    // Match whatever grouped expression matched
    // and then apply the quantifiers after the group itself

    // Return Option::<std::ops::Range>::Some(...) on success
    // Return Option::<std::ops::Range>::None on failure
    fn group_match(&mut self, quantifier: Quantifier) -> Option<Match> {
        let old_pattern = self.pattern.clone();
        self.pattern = old_pattern.children.borrow()[0].borrow().clone();

        let grouped_expression_mactch = {
            // Match a grouped expression

            let match_bound = {
                // Find backtrack entry (in self.backtrack_table) of this group expression
                let table_entry = self
                    .backtrack_table
                    .iter()
                    .find(|entry| entry.index_sequence == self.pattern_index_sequence);
                match table_entry {
                    // This expression matched/backtracked before
                    Some(info) => info.last_match_end.saturating_sub(1),
                    // This expression NEVER matched/backtracked before
                    None => self.target.len(),
                }
            };

            // Start tracking your child
            self.dive();
            // `self.dive()` MUST be called here because it mutates `self.pattern_index_sequence`
            // which is used to find associated entry (in self.backtrack_table) of this group itself
            // Thus calling `self.dive()` before computing `match_bound` makes the search
            // in `self.backtrack_table` always fail

            // A guard to stop matching if inner expression matched the empty string at least once
            // so that Matcher does not loop endlessly matching the empty string at current position
            let mut matched_empty_string = false;

            let start = self.current();
            let mut end = self.current();
            // Keep matching inner expression unless match bound is exceeded
            // or the inner expression matched the empty string at least once
            while let Some(new_match) = self.compute_match() {
                if self.pos > match_bound {
                    // Match bound exceeded while matching inner expression
                    // Roll back to end of most recent successful match
                    self.set_position(end);
                    break;
                }
                if new_match.is_empty() && matched_empty_string {
                    // Stop matching inner expression E when it has
                    // matched the empty string
                    // or Matcher will never stop because it can always
                    // match the empty string at current position
                    break;
                }

                // New match made without exceeding match bound
                // AND the empty string was NOT matched
                // Update match end index of this group expression
                end = new_match.end;
                matched_empty_string = new_match.is_empty();
            }

            // Matched empty range BUT that empty range is NOT the empty string
            // In other words, failed to match even the empty string
            if start == end && !matched_empty_string {
                // Total failure
                match quantifier {
                    // E failed, then so would (E) and (E)+
                    Quantifier::None | Quantifier::OneOrMore => Option::<Match>::None,

                    // E failed, then (E)? and (E)* match the empty string
                    _ => self.empty_expression_match(),
                }
            } else {
                // Matched some string, possibly the empty string
                Some(Match { start, end })
            }

            // Grouped expression computation ends
        };

        // Restore parent pattern to process remaining siblings of current pattern
        self.pattern = old_pattern;
        // Abandon your child
        self.bubble_up();

        grouped_expression_mactch
    }

    // ALTERNATION EXPRESSIONS:
    // (E1|E2|...|E_n) where E1,E2,...,E_n are also expressions
    // for instance, a|b.c|x is an alternation expression

    // HOW TO MATCH AN ALTERNATION EXPRESSION:
    // Match children in order from first to last
    // return the match of the first matching child

    // Return Option::<std::ops::Range>::Some(...) on success
    // Return Option::<std::ops::Range>::None on failure
    fn alternation_match(&mut self) -> Option<Match> {
        // Start tracking your children
        self.dive();

        let old_position = self.current();
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
                    // That is, this functions cleans up its mess after failing to match
                } else {
                    // One of the branches matched
                    // The whole alternation expression has matched
                    // Return that child match
                    break;
                }

                // Current child failed to match
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
    // Match one child after another in order

    // Let E be any child of the concatenation expression

    // If E can backtrack and its has backtracked to start of its last successful match
    // then reset its entry in `self.backtrack_table`

    // Attempt to match subexpression E
    // If match succeeded proceed to match next sibling of E

    // If match failed:
    // check if has at least one preceeding sibling which can backtrack
    // and also it has NOT backtracked to its last successful match end

    // If there is no such sibling then the whole expression fails

    // If there IS AT LEAST one sibling which can backtrack
    // set Matcher position to last successful match start position of that sibling
    // and roll back to that sibling and begin matching again towards the end again

    // Repeat this procees until the last child succeeds
    // or the first subexpression which can backtrack fails to match

    // Return Option::<std::ops::Range>::Some(...) on success
    // Return Option::<std::ops::Range>::None on failure
    fn concatenation_match(&mut self) -> Option<Match> {
        // Start tracking your children
        self.dive();

        let old_position = self.current();
        let old_pattern = self.pattern.clone();

        let concatenation_match = {
            // Match a concatenation expression

            let mut children = self
                .pattern
                .children
                .borrow()
                .iter()
                .map(|rc| {
                    let expr = rc.borrow().clone();
                    // (expression, backtrack table (self.backtrack_table) associated entry index)
                    (expr, Option::<usize>::None)
                })
                .collect::<Vec<_>>();

            let mut match_region_end = self.current();
            let mut child_index = 0usize;

            while child_index < children.len() {
                let (child, table_info_pos) = {
                    let child_entry = &mut children[child_index];
                    (child_entry.0.clone(), child_entry.1)
                };

                // First preceeding sibling which can backtrack
                // AND also has NOT backtracked to its last successful match start
                let prev = children[0..child_index]
                    .iter()
                    .enumerate()
                    .filter(|(idx, (_, table_entry))| {
                        let is_preceeding = *idx < child_index;
                        let supports_backtracking = table_entry.is_some();
                        let can_backtrack_again = supports_backtracking
                            && !self.backtrack_table[table_entry.unwrap()]
                                .backtracked_to_last_match_start;
                        is_preceeding && can_backtrack_again
                    })
                    .map(|(idx, (_, table_entry))| (idx, table_entry))
                    .next_back();

                self.pattern = child.clone();
                if let Some(table_pos) = table_info_pos {
                    // Rust won't allow (self.current()) after (&mut self.backtrack_table)
                    let cur = self.current();
                    let table_entry = &mut self.backtrack_table[table_pos];
                    if prev.is_some() && table_entry.backtracked_to_last_match_start {
                        // This expression backtracked all the way back to start
                        // of its last successful match and it has
                        // a preceeding sibling which can backtrack
                        // Reset its entry in `self.backtrack_table`
                        // to make it usable again
                        table_entry.last_match_start = cur;
                        table_entry.last_match_end = self.target.len();
                        table_entry.backtracked_to_last_match_start = false;
                    }
                }

                // Attempt to match current child
                match self.compute_match() {
                    Some(child_match) => {
                        // Child match succeeded

                        // Record its end index
                        match_region_end = child_match.end;

                        // Store backtrack info if this child supports backtracking
                        let table_entry_index = &mut children[child_index].1;
                        if table_entry_index.is_none() && Self::supports_backtracking(&self.pattern)
                        {
                            // Store backtrack info entry index of this expression
                            let table_pos = self
                                .backtrack_table
                                .binary_search_by(|item| {
                                    item.index_sequence.cmp(&self.pattern_index_sequence)
                                })
                                .unwrap();
                            *table_entry_index = Some(table_pos);
                        }
                    }
                    None => {
                        // Child match failed

                        // Check first preceeding sibling which can backtrack
                        // AND has NOT backtracked to its last successful match start
                        match prev {
                            Some((child_idx, table_entry_idx)) => {
                                // Let processing resume from that sibling
                                child_index = child_idx;

                                let table_entry = {
                                    let table_entry_index = table_entry_idx.unwrap();
                                    &self.backtrack_table[table_entry_index]
                                };
                                // Resume matching from the last successful match start of that sibling
                                self.set_position(table_entry.last_match_start);
                                // Fix subexpressions tracker
                                *self.pattern_index_sequence.last_mut().unwrap() = child_index;
                                continue;
                            }
                            None => {
                                // An item failed to match and none of its
                                // preceeding siblings can backtrack
                                // The whole concatenation expression fails

                                // Restore parent pattern to process remaining siblings of parent pattern
                                self.pattern = old_pattern;

                                // Restore old position
                                self.set_position(old_position);

                                // Abandon your children
                                self.bubble_up();

                                return Option::<Match>::None;
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

impl Iterator for Matcher {
    type Item = Match;

    // Find the next match (non-overlapping with previous match)
    fn next(&mut self) -> Option<Match> {
        // Return Option::<std::ops::Range>::Some(...) on success
        // Return Option::<std::ops::Range>::None on failure

        // Stop if Matcher matched the trailing empty string
        // or it will loop forever because it can always match the trailing empty string
        if self.pos > self.target.len() && self.matched_trailing_empty_string {
            // Target is completely consumed
            // No more matches to compute
            return Option::<Match>::None;
        }

        if self.pos >= self.target.len() {
            // Assume that the trailing empty string is matched
            // even if `self.pattern` can not match the empty string
            self.matched_trailing_empty_string = true;
            // In other words, do NOT attempt to match after this call of `next`
            // just return Option::<Match>::None siganling end of iterator
        }

        // Track root expression
        self.dive();

        // WHY WE NEED A LOOP?
        // Because first match in target string may not start at index 0
        // and hence we need to keep matching until we hit the
        // first successful match or reach end of target
        let mut match_attempt;
        loop {
            match_attempt = self.compute_match();
            // Remove old backtrack info
            self.backtrack_table.clear();
            if match_attempt.is_none() {
                // Last match failed
                if self.has_next() {
                    // Move forward to retry
                    // ADVANCE
                    self.advance();
                } else {
                    // No more characters to process
                    // STOP
                    break;
                }
            } else {
                // Return matched region
                if match_attempt.as_ref().unwrap().is_empty() {
                    // Matched the empty string in current position
                    // Matcher MUST advance or it will loop endlessly
                    // matching the empty string at the same position
                    // because the empty expression can match anywhere
                    self.advance();
                }
                break;
            }
        }

        // Abandon root expression
        self.bubble_up();

        match_attempt
    }
}
