//! This file contains the `TokenType` and `Token` interfaces.

use std::fmt::Display;

/// A enum representing the type of the token.
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Single character tokens
    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// `{`
    OpenBrace,
    /// `}`
    CloseBrace,
    /// `[`
    OpenBrac,
    /// `]`
    CloseBrac,
    /// `;`
    Semi,
    /// `,`
    Comma,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `/`
    Slash,
    /// `*`
    Star,
    /// `=`
    Eq,
    /// `!`
    Bang,
    /// `:`
    Colon,
    /// `>`
    Greater,
    /// `<`
    Lesser,
    /// `?`
    Question,
    /// `&`
    Amp,
    /// `|`
    Pipe,
    // Multiple character tokens
    /// `==`
    EqEq,
    /// `!=`
    BangEq,
    /// `->`
    TArrow,
    /// `=>`
    EArrow,
    /// `&&`
    AmpAmp,
    /// `||`
    PipePipe,

    /// A Keyword
    Keyword(Keyword),

    /// An Identifier
    Identifier(String),

    /// A number
    Number(u64),

    /// A floating point number
    Float(f64),

    /// A string
    String(String),

    /// A character
    Char(char),

    /// End of File
    EOF,
}

/// A enum specifying all of the reserved and used keywords.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Keyword {
    /// `function`
    Fn,
    /// `extern`
    Extern,
    /// `var`
    Var,
    /// `unsafe`
    Unsafe,
    /// `return`
    Return,
    /// `as`
    As,
    /// `if`
    If,
    /// `else`
    Else,
    /// `true`
    True,
    /// `false`
    False,
    /// `inline`
    Inline,
    /// `null`
    Null,
    /// `for`
    For,
    /// `loop`
    Loop,
}

/// A struct representing a token with a type and its location.
#[derive(Debug)]
pub struct Token {
    /// The type of the token.
    pub kind: TokenType,
    /// The position of the token.
    pub position: TokenPosition,
}

impl Token {
    pub(crate) fn new(kind: TokenType, position: TokenPosition) -> Self {
        Self { kind, position }
    }
}

/// The token's position.
#[derive(Debug)]
pub struct TokenPosition {
    /// Start position of the token.
    pub start: usize,
    /// End position of the token.
    pub end: usize,
    /// Line of the token.
    pub line: usize,
}

impl TokenPosition {
    pub(crate) fn new(start: usize, end: usize, line: usize) -> Self {
        Self { start, end, line }
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::Fn => write!(f, "function"),
            Keyword::Extern => write!(f, "extern"),
            Keyword::Var => write!(f, "var"),
            Keyword::Unsafe => write!(f, "unsafe"),
            Keyword::Return => write!(f, "return"),
            Keyword::As => write!(f, "as"),
            Keyword::If => write!(f, "if"),
            Keyword::Else => write!(f, "else"),
            Keyword::True => write!(f, "true"),
            Keyword::False => write!(f, "false"),
            Keyword::Inline => write!(f, "inline"),
            Keyword::Null => write!(f, "null"),
            Keyword::For => write!(f, "for"),
            Keyword::Loop => write!(f, "loop"),
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::OpenParen => write!(f, "("),
            TokenType::CloseParen => write!(f, ")"),
            TokenType::OpenBrace => write!(f, "{{"),
            TokenType::CloseBrace => write!(f, "}}"),
            TokenType::OpenBrac => write!(f, "["),
            TokenType::CloseBrac => write!(f, "]"),
            TokenType::Semi => write!(f, ";"),
            TokenType::Comma => write!(f, ","),
            TokenType::Plus => write!(f, "+"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Star => write!(f, "*"),
            TokenType::Eq => write!(f, "="),
            TokenType::Bang => write!(f, "!"),
            TokenType::Colon => write!(f, ":"),
            TokenType::Greater => write!(f, ">"),
            TokenType::Lesser => write!(f, "<"),
            TokenType::Question => write!(f, "?"),
            TokenType::Amp => write!(f, "&"),
            TokenType::Pipe => write!(f, "|"),
            TokenType::EqEq => write!(f, "=="),
            TokenType::BangEq => write!(f, "!="),
            TokenType::TArrow => write!(f, "->"),
            TokenType::EArrow => write!(f, "=>"),
            TokenType::AmpAmp => write!(f, "&&"),
            TokenType::PipePipe => write!(f, "||"),
            TokenType::Keyword(keyword) => write!(f, "{}", keyword),
            TokenType::Identifier(identifier) => write!(f, "{}", identifier),
            TokenType::Number(number) => write!(f, "{}", number),
            TokenType::Float(float) => write!(f, "{}", float),
            TokenType::String(string) => write!(f, "{}", string),
            TokenType::Char(char) => write!(f, "{}", char),
            TokenType::EOF => write!(f, "EOF"),
        }
    }
}
