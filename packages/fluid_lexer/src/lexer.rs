//! This file contains the actual lexer implementation, the `Lexer` interface.

use std::process;

use fluid_error::{lex_error, Error, ErrorType};

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
    pub fn run(&mut self) -> Vec<Token> {
        let mut tokens = vec![];
        let mut panicked = false;

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
                    panicked = true;
                    err.flush();

                    advance!(self);
                }
            }
        }

        // Exit if the lexer has panicked
        if panicked {
            process::exit(1);
        }

        tokens
    }

    /// Returns the current character.
    /// TODO: Remove every usage of current_char and replace it with peek and handle if there is an expected char.
    fn current_char(&self) -> char {
        self.code.chars().nth(self.index).unwrap()
    }

    /// Peek the current character.
    fn peek(&self) -> Option<char> {
        self.code.chars().nth(self.index)
    }

    /// Scans the next character a new `Token`.
    /// It will fail if an illegal character is encountered. Thus, in that case it will result in returning an `Error`.
    pub fn get_next_token(&mut self) -> Result<Token, Error> {
        self.skip_whitespaces_and_comments();

        if self.peek().is_none() {
            // Return the EOF token if the lexer has reached at the end of the file.
            return Ok(self.new_token(TokenType::EOF));
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
            _ => lex_error!(
                self,
                ["Roses are red, violets are blue. Unexpected character `", self.current_char(), "`", " on line ", self.line.to_string()]
            ),
        }
    }

    /// Collect a string.
    fn collect_str(&mut self) -> Result<Token, Error> {
        // Advance '"'
        advance!(self);

        let mut string = String::new();

        while self.peek().is_some() && self.current_char() != '"' {
            string.push(self.current_char());
            advance!(self);
        }

        // Advance '"'
        advance!(self);

        Ok(self.new_token(TokenType::String(string)))
    }

    /// Collect a character
    fn collect_char(&mut self) -> Result<Token, Error> {
        // Advance "'"
        advance!(self);

        let char_v = self.current_char();

        // Advance the char and "'"
        advance!(self);
        advance!(self);

        Ok(self.new_token(TokenType::Char(char_v)))
    }

    /// Collect an identifier.
    fn collect_id(&mut self) -> Option<Token> {
        let mut id = String::new();

        while self.peek().is_some() && (self.current_char().is_alphabetic() || self.current_char() == '_') {
            id.push(self.current_char());
            advance!(self);
        }

        if id != String::new() {
            match id.as_str() {
                "function" => Some(self.new_token(TokenType::Keyword(Keyword::Fn))),
                "extern" => Some(self.new_token(TokenType::Keyword(Keyword::Extern))),

                "return" => Some(self.new_token(TokenType::Keyword(Keyword::Return))),
                "var" => Some(self.new_token(TokenType::Keyword(Keyword::Var))),

                "as" => Some(self.new_token(TokenType::Keyword(Keyword::As))),
                "unsafe" => Some(self.new_token(TokenType::Keyword(Keyword::Unsafe))),
                "inline" => Some(self.new_token(TokenType::Keyword(Keyword::Inline))),

                "null" => Some(self.new_token(TokenType::Keyword(Keyword::Null))),

                "if" => Some(self.new_token(TokenType::Keyword(Keyword::If))),
                "else" => Some(self.new_token(TokenType::Keyword(Keyword::Else))),

                "true" => Some(self.new_token(TokenType::Keyword(Keyword::True))),
                "false" => Some(self.new_token(TokenType::Keyword(Keyword::False))),

                "for" => Some(self.new_token(TokenType::Keyword(Keyword::For))),
                "loop" => Some(self.new_token(TokenType::Keyword(Keyword::Loop))),

                _ => Some(self.new_token(TokenType::Identifier(id))),
            }
        } else {
            None
        }
    }

    /// Collect a number.
    fn collect_number(&mut self) -> Option<Token> {
        let mut number = String::new();
        let mut typee = "number";

        while self.peek().is_some() && self.current_char().is_ascii_digit() {
            number.push(self.current_char());
            advance!(self);

            if self.peek().is_some() && self.current_char() == '.' {
                typee = "float";
                number.push('.');

                advance!(self);
            }
        }

        if number != String::new() {
            match typee {
                "number" => return Some(self.new_token(TokenType::Number(number.parse().unwrap()))),
                "float" => return Some(self.new_token(TokenType::Float(number.parse().unwrap()))),
                _ => unreachable!(),
            }
        }

        None
    }

    /// Skip all of the white spaces and comments.
    fn skip_whitespaces_and_comments(&mut self) {
        while self.peek().is_some() {
            match self.current_char() {
                '\n' => {
                    advance!(self);

                    self.line += 1;
                }
                '\r' => advance!(self),
                '/' => {
                    if self.peek().is_some() && self.current_char() == '/' {
                        while self.peek().is_some() {
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

    /// Create a token with its mentioned type
    fn new_token(&self, kind: TokenType) -> Token {
        let index = self.index;
        let line = self.line;

        Token { kind, index, line }
    }
}
