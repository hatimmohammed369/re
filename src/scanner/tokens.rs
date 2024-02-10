// disable some annoying warnings
#[allow(dead_code)]
#[allow(clippy::let_and_return)]
// enable pretty-printing if needed
#[derive(Debug, PartialEq)]
pub enum TokenName {
    // Token types

    // *SPECIAL
    // indicator of places like:
    // "|..." "...|" "...()..." "...(|)..." "" (an empty string)
    EmptyString,
    // a non-metacharacter and not an escaped metacharacter
    Character { value: char },

    // *METACHARACTERS
    LeftParen,  // (
    RightParen, // )
    Pipe,       // |, alternation operator (E1|E2|...|E_n)
    Mark,       // ?, match zero or one occurrence of previous expression
    Star,       // *, match zero or more occurrences of previous expression
    Plus,       // +, match zero or more occurrences of previous expression
    Dot,        // ., match any single character even newline `\n`

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
