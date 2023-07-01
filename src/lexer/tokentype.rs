#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    QuestionMark,
    Colon,

    // One or two character tokens.
    Bang,
    BangEqual,
    Assign,
    Equals,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords,
    And,
    Class,
    Else,
    False,
    DefFn,
    For,
    If,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Let,
    None,
    While,
    Break,

    EOF,
}
