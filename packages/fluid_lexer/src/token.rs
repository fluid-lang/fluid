//! This file contains the `TokenType` and `Token` interfaces.

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
    /// The index of the token.
    pub index: usize,
    /// The line of the token.
    pub line: usize,
}
