//! This file contains all of the unit tests for the lexer.

use crate::{Keyword, Lexer, TokenType};

/// Test a function declaration
#[test]
fn test_function() {
    let source = "
        function hello() {
            print(\"World\");
        }
    ";

    let filename = "<test>";

    let mut lexer = Lexer::new(source, filename);
    let tokens = lexer.run().into_iter().map(|token| token.kind).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            TokenType::Keyword(Keyword::Fn),
            TokenType::Identifier(String::from("hello")),
            TokenType::OpenParen,
            TokenType::CloseParen,
            TokenType::OpenBrace,
            TokenType::Identifier(String::from("print")),
            TokenType::OpenParen,
            TokenType::String(String::from("World")),
            TokenType::CloseParen,
            TokenType::Semi,
            TokenType::CloseBrace,
            TokenType::EOF
        ]
    );
}
