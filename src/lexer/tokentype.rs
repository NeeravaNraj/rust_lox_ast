#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    RightBracket,
    LeftBracket,
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
    PlusEqual,
    MinusEqual,
    SlashEqual,
    StarEqual,
    PlusPlus,
    MinusMinus,
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
    DefLambda,
    For,
    If,
    Elif,
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
    Continue,
    Static,

    EOF,
}
