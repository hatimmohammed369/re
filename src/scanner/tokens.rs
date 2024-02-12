// disable some annoying warnings
#[allow(dead_code)]
#[allow(clippy::let_and_return)]
// enable pretty-printing if needed
#[derive(Debug, PartialEq)]
pub enum TokenName {
    // Token types (names)
    // When we say `an Empty token` we mean a Token object
    // whose `name` field is set to `TokenName::Empty`

    // *SPECIAL
    // indicator of places like:
    // "|..." "...|" "...()..." "...(|)..." "" (an empty string)
    Empty,
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

// Scanner generates `Tokens` which are a atoms of regular expressions
// Token is identified by two properties:
// name    : a variant of TokenName
// position: usize integer indicating where this Token begins inside source string given to the
// scanner
// The scanner just splits the pattern string for the parser

// enable pretty-printing if needed
#[derive(Debug)]
pub struct Token {
    // What kind this token is?
    pub name: TokenName,
    // index in source string
    pub position: usize,
}
