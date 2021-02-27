//! This file contains all of the unit tests for the lexer.

use crate::{Keyword, Lexer, Token, TokenType};

#[inline]
fn get_token_type(tokens: Vec<Token>) -> Vec<TokenType> {
    tokens.into_iter().map(|token| token.kind).collect::<Vec<_>>()
}

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
    let tokens = get_token_type(lexer.run().unwrap());

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

/// A test comment.
#[test]
fn test_comment() {
    let source = "
        // Hello World!
        /*
            This is soo cool!

            * !
            * !
        */
    ";

    let filename = "<test>";

    let mut lexer = Lexer::new(source, filename);
    let tokens = get_token_type(lexer.run().unwrap());

    assert_eq!(tokens, vec![TokenType::EOF]);
}

/// String escape tests.
#[test]
fn string_test() {
    let source = "
        \"World\"

        \"World\\n\"
        \"World\\t\"
        \"World\\r\"
        \"World\\0\"
        \"\\x48\\x65\\x6c\\x6c\\x6f World\"
        \"I \\u{1F496} World\"
        \"Hello \\b World\"

        \"World\\\"\"
    ";

    let filename = "<test>";

    let mut lexer = Lexer::new(source, filename);
    let tokens = get_token_type(lexer.run().unwrap());

    assert_eq!(
        tokens,
        vec![
            TokenType::String(String::from("World")),
            TokenType::String(String::from("World\n")),
            TokenType::String(String::from("World\t")),
            TokenType::String(String::from("World\r")),
            TokenType::String(String::from("World\0")),
            TokenType::String(String::from("Hello World")),
            TokenType::String(String::from("I \u{1F496} World")), // 💖, Unicode scalar U+1F496
            TokenType::String(String::from("Hello \x08 World")),
            TokenType::String(String::from("World\"")),
            TokenType::EOF
        ]
    );
}

/// Test shebang.
#[test]
fn test_shebang() {
    let source = "
        #!/usr/bin/env fluid run
    ";

    let filename = "<test>";

    let mut lexer = Lexer::new(source, filename);
    let tokens = get_token_type(lexer.run().unwrap());

    assert_eq!(tokens, vec![TokenType::EOF]);
}
