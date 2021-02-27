//! This file contains the actual lexer implementation, the `Lexer` interface.

use fluid_error::{AnnotationType, Diagnostic, DiagnosticBuilder, Slice, SourceAnnotation};

use crate::advance;
use crate::token::*;

/// Returns true if the character is considered a whitespace.
fn is_whitespace(char: char) -> bool {
    matches!(
        char,
        // Usual ASCII suspects
        '\u{0009}'   // \t
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0020}' // space

        // Next line from latin1
        | '\u{0085}'

        // Bidi markers
        | '\u{200E}' // Left to right => mark
        | '\u{200F}' // Right to left => mark

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // Line separator
        | '\u{2029}' // Paragraph separator
    )
}

/// Returns true if its a valid continuation of an identifer.
fn is_valid_continuation_of_identifier(char: char) -> bool {
    char.is_ascii_alphabetic() || char.is_ascii_digit() || matches!(char, '_')
}

/// Returns true if its a valid start of an identifier.
fn is_valid_start_of_identifier(char: char) -> bool {
    char.is_ascii_alphabetic() || matches!(char, '_')
}

/// Contains the internal state while processing a Fluid file.
#[derive(Debug)]
pub struct Lexer {
    /// The name of the file we are going to scan.
    pub file: String,
    /// The contents of the file that we are going to scan.
    pub code: String,
    /// The current index.
    index: usize,
    /// The current position.
    position: usize,
    /// The current line number.
    line: usize,
}

impl Lexer {
    /// Create a new instance of the lexer.
    pub fn new(code: impl Into<String>, file: impl Into<String>) -> Self {
        let code = code.into();
        let file = file.into();

        let position = 0;
        let index = 0;
        let line = 1;

        Self { file, code, index, position, line }
    }

    /// Runs `self.get_next_token()` until the current character is not EOF.
    /// After it has encountered EOF it appends the EOF Token.
    /// Then it returns all of the collected tokens.
    pub fn run(&mut self) -> Result<Vec<Token>, Vec<Diagnostic>> {
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
                    // Enter panic mode if there is an Diagnostic.
                    errors.push(err);

                    self.advance();
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

    /// Scans the next character and return a new `Token`. The source end is indicated by token.EOF.
    /// It will fail if an illegal character is encountered. Thus, in that case it will result in returning a `Diagnostic`.
    pub fn get_next_token(&mut self) -> Result<Token, Diagnostic> {
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
    fn collect_str(&mut self) -> Result<Token, Diagnostic> {
        let index_start = self.index;
        let line_start = self.line;

        // Advance '"'
        self.advance();

        let mut string = String::new();

        while !self.is_eof() && self.current_char() != '"' {
            if self.current_char() == '\n' {
                self.line += 1;
            }

            string.push(self.current_char());
            self.advance();
        }

        if self.is_eof() {
            return Err(self
                .make_error("unterminated string literal", "E0002")
                .push_slice(
                    Slice::new()
                        .set_line_start(line_start)
                        .set_line_end(self.line)
                        .push_annotation(SourceAnnotation::new().set_kind(AnnotationType::Error).set_range(index_start..self.index)),
                )
                .build());
        }

        // Advance '"'
        self.advance();

        Ok(self.new_token(TokenType::String(string), index_start, self.index))
    }

    /// Collect a character
    fn collect_char(&mut self) -> Result<Token, Diagnostic> {
        let start = self.index;

        // Advance "'"
        self.advance();

        let char_v = self.current_char();

        // Advance the char.
        self.advance();

        if self.is_eof() || self.current_char() != '\'' {
            let err = Err(self
                .make_error("unterminated character literal", "E0002")
                .push_slice(
                    Slice::new()
                        .set_line_start(self.line)
                        .push_annotation(SourceAnnotation::new().set_kind(AnnotationType::Error).set_range(start..self.index)),
                )
                .build());

            if !self.is_eof() && self.current_char() != '\'' {
                self.advance();
            }

            return err;
        }

        // Advance "'"
        self.advance();

        Ok(self.new_token(TokenType::Char(char_v), start, self.index))
    }

    /// Collect an identifier.
    fn collect_id(&mut self) -> Option<Token> {
        if is_valid_start_of_identifier(self.current_char()) {
            let start = self.index;
            let mut id = String::new();

            id.push(self.current_char());
            self.advance();

            while !self.is_eof() && is_valid_continuation_of_identifier(self.current_char()) {
                id.push(self.current_char());

                self.advance();
            }

            // Get the substring.
            let id = id.as_str();

            match id {
                "function" => Some(self.new_token(TokenType::Keyword(Keyword::Fn), start, self.index)),
                "extern" => Some(self.new_token(TokenType::Keyword(Keyword::Extern), start, self.index)),

                "return" => Some(self.new_token(TokenType::Keyword(Keyword::Return), start, self.index)),
                "var" => Some(self.new_token(TokenType::Keyword(Keyword::Var), start, self.index)),

                "as" => Some(self.new_token(TokenType::Keyword(Keyword::As), start, self.index)),
                "unsafe" => Some(self.new_token(TokenType::Keyword(Keyword::Unsafe), start, self.index)),

                "null" => Some(self.new_token(TokenType::Keyword(Keyword::Null), start, self.index)),

                "if" => Some(self.new_token(TokenType::Keyword(Keyword::If), start, self.index)),
                "else" => Some(self.new_token(TokenType::Keyword(Keyword::Else), start, self.index)),

                "true" => Some(self.new_token(TokenType::Keyword(Keyword::True), start, self.index)),
                "false" => Some(self.new_token(TokenType::Keyword(Keyword::False), start, self.index)),

                "for" => Some(self.new_token(TokenType::Keyword(Keyword::For), start, self.index)),
                "loop" => Some(self.new_token(TokenType::Keyword(Keyword::Loop), start, self.index)),

                _ => Some(self.new_token(TokenType::Identifier(id.into()), start, self.index)),
            }
        } else {
            None
        }
    }

    /// Collect a number.
    fn collect_number(&mut self) -> Option<Token> {
        let start = self.index;
        let mut number = String::new();
        let mut typee = "number";

        while !self.is_eof() && self.current_char().is_ascii_digit() {
            number.push(self.current_char());
            self.advance();

            if !self.is_eof() && self.current_char() == '.' {
                typee = "float";
                number.push('.');

                self.advance();
            }
        }

        if number != String::new() {
            match typee {
                "number" => return Some(self.new_token(TokenType::Number(number.parse().unwrap()), start, self.index)),
                "float" => return Some(self.new_token(TokenType::Float(number.parse().unwrap()), start, self.index)),
                _ => unreachable!(),
            }
        }

        None
    }

    /// Skip all of the white spaces and comments.
    fn skip_whitespaces_and_comments(&mut self) {
        while !self.is_eof() {
            if is_whitespace(self.current_char()) {
                self.advance();

                continue;
            }

            match self.current_char() {
                '\n' => {
                    self.advance();

                    self.line += 1;
                    self.index = 0;
                }
                '/' => {
                    // Advance '/'
                    self.advance();

                    if !self.is_eof() && self.current_char() == '/' {
                        while !self.is_eof() {
                            if self.current_char() == '\n' {
                                break;
                            }

                            self.advance();
                        }
                    } else if !self.is_eof() && self.current_char() == '*' {
                        self.advance();

                        loop {
                            if self.is_eof() {
                                // TODO: Return error.
                                break;
                            }

                            if self.current_char() == '*' && self.next_char() == '/' {
                                // Advance '*'
                                self.advance();
                                // Advance '/'
                                self.advance();

                                break;
                            }

                            self.advance();
                        }
                    }
                }
                _ => break,
            }
        }
    }

    /// Make a error with a message, code.
    fn make_error(&self, message: impl Into<String>, code: impl Into<String>) -> DiagnosticBuilder {
        DiagnosticBuilder::new()
            .set_source(&self.code)
            .set_origin(&self.file)
            .set_type(AnnotationType::Error)
            .set_message(message.into())
            .set_code(code.into())
    }

    /// Throw a unexpected char error.
    #[inline]
    fn throw_unexpected_char(&mut self) -> Diagnostic {
        let err = self
            .make_error("illegal character encountered", "E0001")
            .push_slice(
                Slice::new().set_line_start(self.line).push_annotation(
                    SourceAnnotation::new()
                        .set_kind(AnnotationType::Error)
                        .set_label("unknown character")
                        .set_range(self.index..self.index + 1),
                ),
            )
            .build();

        self.advance();

        err
    }

    /// Advance to the next character.
    fn advance(&mut self) {
        self.position += 1;
        self.index += 1;
    }

    /// Returns the current character.
    #[inline]
    fn current_char(&self) -> char {
        self.code.chars().nth(self.position).unwrap()
    }

    /// Returns the next character.
    #[inline]
    fn next_char(&self) -> char {
        self.code.chars().nth(self.position + 1).unwrap()
    }

    /// Check if lexer has reached the EOF (End of File)
    #[inline]
    fn is_eof(&self) -> bool {
        self.code.chars().nth(self.position).is_none()
    }

    /// Create a token with its mentioned type
    fn new_token(&self, kind: TokenType, pos_start: usize, pos_end: usize) -> Token {
        let position = TokenPosition::new(pos_start, pos_end, self.line);

        Token::new(kind, position)
    }
}
