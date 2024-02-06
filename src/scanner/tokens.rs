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

pub struct Token {
    name: TokenName,
    position: usize,
}
