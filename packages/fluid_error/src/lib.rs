#![deny(unsafe_code)]

use ansi_term::Colour::{Fixed, Red};

#[derive(Debug)]
pub enum ErrorType {
    Lexer,
}

#[derive(Debug)]
pub struct Error {
    source: String,
    line: usize,
    error_type: ErrorType,
    message: String,
    filename: String,
}

impl Error {
    pub fn new(source: &str, filename: &str, line: usize, error_type: ErrorType, message: String) -> Self {
        let source = source.split("\n").collect::<Vec<_>>()[line - 1].into();
        let filename = filename.to_owned();

        Self {
            source,
            line,
            error_type,
            message,
            filename,
        }
    }

    pub fn flush(&self) {
        eprintln!(
            "{}: {}\n{}\n{}\n",
            Red.bold().paint(format!("error[{}]", self.error_type)),
            self.message,
            format!("  {} Source: {}", Fixed(238).paint("───>"), self.filename),
            format!("{} {} {}", Fixed(244).paint(format!("{:4}", self.line)), Fixed(238).paint("│"), self.source)
        );
    }
}

#[macro_export]
macro_rules! lex_error {
    ($self:ident, [$($arg:expr),*]) => {{
        let mut message = String::new();

        $(
            let arg: String = $arg.into();

            message.push_str(arg.as_str());
        )*

        Err(Error::new(&$self.code, &$self.file, $self.line, ErrorType::Lexer, message))
    }};
}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ErrorType::Lexer => write!(f, "lex"),
        }
    }
}
