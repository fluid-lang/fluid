//! This file contains the actual lexer implementation, the `Lexer` interface.

use fluid_error::Error;

use crate::advance;
use crate::token::*;

/// Contains the internal state while processing a Fluid file.
#[derive(Debug)]
pub struct Lexer {
    /// The name of the file we are going to scan.
    pub file: String,
    /// The contents of the file that we are going to scan.
    pub code: String,
    /// The current index.
    pub index: usize,
    /// The current line number.
    pub line: usize,
}

impl Lexer {
    /// Create a new instance of the lexer.
    pub fn new<S: Into<String>>(code: S, file: S) -> Self {
        let code = code.into();
        let file = file.into();

        let index = 0;
        let line = 1;

        Self { file, code, index, line }
    }

    /// Runs `self.get_next_token()` until the current character is not EOF.
    /// After it has encountered EOF it appends the EOF Token.
    /// Then it returns all of the collected tokens.
    pub fn run(&mut self) -> Result<Vec<Token>, Vec<Error>> {
        let mut tokens = vec![];
        let mut errors = vec![];

        loop {
            match self.get_next_token() {
                Ok(token) => {
                    if token.kind == TokenType::EOF {
                        tokens.push(token);

                        // If the lexer has reached at the end of file then stop scanning for new tokens.
                        break;
                    } else {
                        tokens.push(token);
                    }
                }

                Err(err) => {
                    // Enter panic mode if there is an error.
                    errors.push(err);

                    advance!(self);
                }
            }
        }

        // Exit if the lexer has panicked.
        if errors.is_empty() {
            Ok(tokens)
        } else {
            Err(errors)
        }
    }

    /// Returns the current character.
    fn current_char(&self) -> char {
        self.code.chars().nth(self.index).unwrap()
    }

    /// Check if lexer has reached the EOF (End of File)
    fn is_eof(&self) -> bool {
        self.code.chars().nth(self.index).is_none()
    }

    /// Scans the next character a new `Token`.
    /// It will fail if an illegal character is encountered. Thus, in that case it will result in returning an `Error`.
    pub fn get_next_token(&mut self) -> Result<Token, Error> {
        self.skip_whitespaces_and_comments();

        if self.is_eof() {
            // Return the EOF token if the lexer has reached at the end of the file.
            return Ok(self.new_token(TokenType::EOF, self.index, self.index));
        }

        if let Some(token) = self.collect_id() {
            return Ok(token);
        } else if let Some(token) = self.collect_number() {
            return Ok(token);
        }

        match self.current_char() {
            '(' => advance!(self, TokenType::OpenParen),
            ')' => advance!(self, TokenType::CloseParen),
            '{' => advance!(self, TokenType::OpenBrace),
            '}' => advance!(self, TokenType::CloseBrace),
            '[' => advance!(self, TokenType::OpenBrac),
            ']' => advance!(self, TokenType::CloseBrac),
            ';' => advance!(self, TokenType::Semi),
            ',' => advance!(self, TokenType::Comma),
            '+' => advance!(self, TokenType::Plus),
            '/' => advance!(self, TokenType::Slash),
            '*' => advance!(self, TokenType::Star),
            ':' => advance!(self, TokenType::Colon),
            '>' => advance!(self, TokenType::Greater),
            '<' => advance!(self, TokenType::Lesser),
            '?' => advance!(self, TokenType::Question),
            '-' => advance!(self, ['>' => TokenType::TArrow], TokenType::Minus),
            '!' => advance!(self, ['=' => TokenType::BangEq], TokenType::Bang),
            '&' => advance!(self, ['&' => TokenType::AmpAmp], TokenType::Amp),
            '|' => advance!(self, ['|' => TokenType::PipePipe], TokenType::Pipe),
            '=' => advance!(self, ['=' => TokenType::EqEq, '>' => TokenType::EArrow], TokenType::Eq),
            '"' => self.collect_str(),
            '\'' => self.collect_char(),
            _ => Err(self.throw_unexpected_char()),
        }
    }

    /// Collect a string.
    fn collect_str(&mut self) -> Result<Token, Error> {
        let index = self.index;

        // Advance '"'
        advance!(self);

        let mut string = String::new();

        while !self.is_eof() && self.current_char() != '"' {
            string.push(self.current_char());
            advance!(self);
        }

        // Advance '"'
        advance!(self);

        Ok(self.new_token(TokenType::String(string), index, self.index))
    }

    /// Collect a character
    fn collect_char(&mut self) -> Result<Token, Error> {
        let index = self.index;

        // Advance "'"
        advance!(self);

        let char_v = self.current_char();

        // Advance the char and "'"
        advance!(self);
        advance!(self);

        Ok(self.new_token(TokenType::Char(char_v), index, self.index))
    }

    /// Collect an identifier.
    fn collect_id(&mut self) -> Option<Token> {
        let index = self.index;
        let mut id = String::new();

        while !self.is_eof() && (self.current_char().is_alphabetic() || self.current_char() == '_') {
            id.push(self.current_char());
            advance!(self);
        }

        if id != String::new() {
            match id.as_str() {
                "function" => Some(self.new_token(TokenType::Keyword(Keyword::Fn), index, self.index)),
                "extern" => Some(self.new_token(TokenType::Keyword(Keyword::Extern), index, self.index)),

                "return" => Some(self.new_token(TokenType::Keyword(Keyword::Return), index, self.index)),
                "var" => Some(self.new_token(TokenType::Keyword(Keyword::Var), index, self.index)),

                "as" => Some(self.new_token(TokenType::Keyword(Keyword::As), index, self.index)),
                "unsafe" => Some(self.new_token(TokenType::Keyword(Keyword::Unsafe), index, self.index)),

                "null" => Some(self.new_token(TokenType::Keyword(Keyword::Null), index, self.index)),

                "if" => Some(self.new_token(TokenType::Keyword(Keyword::If), index, self.index)),
                "else" => Some(self.new_token(TokenType::Keyword(Keyword::Else), index, self.index)),

                "true" => Some(self.new_token(TokenType::Keyword(Keyword::True), index, self.index)),
                "false" => Some(self.new_token(TokenType::Keyword(Keyword::False), index, self.index)),

                "for" => Some(self.new_token(TokenType::Keyword(Keyword::For), index, self.index)),
                "loop" => Some(self.new_token(TokenType::Keyword(Keyword::Loop), index, self.index)),

                _ => Some(self.new_token(TokenType::Identifier(id), index, self.index)),
            }
        } else {
            None
        }
    }

    /// Collect a number.
    fn collect_number(&mut self) -> Option<Token> {
        let index = self.index;
        let mut number = String::new();
        let mut typee = "number";

        while !self.is_eof() && self.current_char().is_ascii_digit() {
            number.push(self.current_char());
            advance!(self);

            if !self.is_eof() && self.current_char() == '.' {
                typee = "float";
                number.push('.');

                advance!(self);
            }
        }

        if number != String::new() {
            match typee {
                "number" => return Some(self.new_token(TokenType::Number(number.parse().unwrap()), index, self.index)),
                "float" => return Some(self.new_token(TokenType::Float(number.parse().unwrap()), index, self.index)),
                _ => unreachable!(),
            }
        }

        None
    }

    /// Skip all of the white spaces and comments.
    fn skip_whitespaces_and_comments(&mut self) {
        while !self.is_eof() {
            match self.current_char() {
                '\n' => {
                    advance!(self);

                    self.line += 1;
                }
                '\r' => advance!(self),
                '/' => {
                    if !self.is_eof() && self.current_char() == '/' {
                        while !self.is_eof() {
                            if self.current_char() == '\n' {
                                break;
                            }

                            advance!(self);
                        }
                    } else {
                        advance!(self);
                    }
                }
                '\t' => advance!(self),
                ' ' => advance!(self),
                _ => break,
            }
        }
    }

    /// Throw a unexpected char error.
    #[inline]
    fn throw_unexpected_char(&mut self) -> Error {
        todo!()
    }

    /// Create a token with its mentioned type
    fn new_token(&self, kind: TokenType, pos_start: usize, pos_end: usize) -> Token {
        let position = TokenPosition::new(pos_start, pos_end, self.line);

        Token::new(kind, position)
    }
}
