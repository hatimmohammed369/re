// disable some annoying warnings
#[allow(dead_code)]
#[allow(clippy::let_and_return)]
// enable pretty-printing if needed
#[derive(Debug)]
pub enum TokenName {
    // Token types

    // *SPECIAL
    // indicator of places like:
    // "|..." "...|" "...()..." "...(|)..." ""
    EmptyString,
    // a non-metacharacter and not an escaped metacharacter
    Character { value: char },

    // *METACHARACTERS
    LeftParen,  // (
    RightParen, // )
    Pipe,       // |
    Mark,       // ?
    Star,       // *
    Plus,       // +
    Dot,        // .

    // *ESCAPED METACHARACTER
    // used to match a literal metacharacter
    EscapedSlash,      // \\
    EscapedLeftParen,  // \(
    EscapedRightParen, // \)
    EscapedPipe,       // \|
    EscapedMark,       // \?
    EscapedStar,       // \*
    EscapedPlus,       // \+
    EscapedDot,        // \.
}

// enable pretty-printing if needed
#[derive(Debug)]
pub struct Token {
    pub name: TokenName,
    // index in source string
    pub position: usize,
}
