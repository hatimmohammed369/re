// Use a parsed regular expression to match against strings

use crate::parser::{parse, syntax_tree::Regexp};

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

// Co-ordinator of the matching process
#[allow(dead_code)]
pub struct Matcher {
    // Parsed pattern
    pattern: Regexp,
    // String on which the search (pattern matching) is done
    target: String,
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
        let target = String::from(target);
        let current = 0;
        let matched_trailing_empty_string = false;
        Ok(Matcher {
            pattern,
            target,
            current,
            matched_trailing_empty_string,
        })
    }

    // Assign a new target to match on
    pub fn assign_match_target(&mut self, target: &str) {
        self.target = String::from(target);
        self.current = 0;
        self.matched_trailing_empty_string = false;
    }
}
