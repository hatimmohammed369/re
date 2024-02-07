#[allow(dead_code)]
#[allow(clippy::let_and_return)]
#[derive(Debug)]
pub enum TokenName {
    EmptyString,
    Character { value: char },
    LeftParen,         // (
    RightParen,        // )
    Pipe,              // |
    Mark,              // ?
    Star,              // *
    Plus,              // +
    Dot,               // .
    EscapedSlash,      // \\
    EscapedLeftParen,  // \(
    EscapedRightParen, // \)
    EscapedPipe,       // \|
    EscapedMark,       // \?
    EscapedStar,       // \*
    EscapedPlus,       // \+
    EscapedDot,        // \.
}

#[derive(Debug)]
pub struct Token {
    pub name: TokenName,
    pub position: usize,
}
