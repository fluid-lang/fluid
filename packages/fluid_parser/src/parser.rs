//! This file contains the parser implementation, the `Parser` interface.
//!
//! Here is the binary precedence table for reference.
//!
//! Assignment = 1                 =          (1 case) \
//! Or = 2                         ||         (1 case) \
//! And = 3                        &&         (1 case) \
//! Equality = 4                   ==         (1 case) \
//! Comparison = 5                 <, >       (2 cases) \
//! Term = 6                       +, -       (2 cases) \
//! Factor = 7                     *, /       (2 cases) \
//!

use fluid_lexer::{Keyword, Token, TokenType};

use crate::advance;
use crate::ast::*;

/// Contains the internal state while processing the tokens provided by the lexer.
#[derive(Debug)]
pub struct Parser {
    /// The tokens generated by the lexer.
    pub tokens: Vec<Token>,
    /// The current index of the parser.
    pub index: usize,
}

impl Parser {
    /// Create a new instance of the parser.
    pub fn new(tokens: Vec<Token>) -> Self {
        let index = 0;

        Self { tokens, index }
    }

    /// Run the parser.
    pub fn run(&mut self) -> Vec<Statement> {
        let mut ast = vec![];

        while self.index < self.tokens.len() && *self.peek() != TokenType::EOF {
            ast.push(self.parse_statement());
        }

        ast
    }

    /// Parse a function definition.
    fn parse_fn_def(&mut self) -> Statement {
        let prototype = self.parse_proto();
        let body = self.parse_block();

        let func = Function { prototype, body };

        Statement::Declaration(Box::new(Declaration::Function(func)))
    }

    /// Parse a type.
    ///
    /// Types: \
    ///     => void \
    ///     => number \
    ///     => float \
    ///     => string \
    ///     => $tuple($(type),*)
    ///
    /// TODO: `void` should be a type alais for `()` an empty tuple.
    fn parse_type(&mut self) -> Type {
        let kind = match self.peek() {
            TokenType::Identifier(kind) => match kind.as_str() {
                "void" => Type::Void,
                "number" => Type::Number,
                "float" => Type::Float,
                "string" => Type::String,
                _ => unimplemented!(),
            },
            TokenType::OpenParen => self.parse_tuple_type(),

            _ => panic!("Expected a type."),
        };

        advance!(self);

        kind
    }

    /// Parse a tuple type.
    ///
    /// $tuple($(type),*)
    fn parse_tuple_type(&mut self) -> Type {
        todo!()
    }

    /// Parse function prototype.
    fn parse_proto(&mut self) -> Prototype {
        self.expect(TokenType::Keyword(Keyword::Fn));

        let name = advance!(self => TokenType::Identifier);
        let mut args = vec![];

        self.expect(TokenType::OpenParen);

        while *self.peek() != TokenType::CloseParen {
            let arg_name = advance!(self => TokenType::Identifier);

            self.expect(TokenType::Colon);

            let arg_type = self.parse_type();

            if *self.peek() != TokenType::CloseParen {
                self.expect(TokenType::Comma);
            }

            args.push(Arg { name: arg_name, typee: arg_type });
        }

        self.expect(TokenType::CloseParen);

        let return_type;

        if *self.peek() == TokenType::TArrow {
            self.expect(TokenType::TArrow);

            return_type = self.parse_type();
        } else {
            return_type = Type::default();
        }

        Prototype { name, args, return_type }
    }

    /// Parse a extern definition
    fn parse_extern(&mut self) -> Statement {
        let mut externs = vec![];

        self.expect(TokenType::Keyword(Keyword::Extern));
        self.expect(TokenType::OpenBrace);

        while *self.peek() != TokenType::CloseBrace {
            externs.push(self.parse_proto());
            self.expect(TokenType::Semi);
        }

        self.expect(TokenType::CloseBrace);

        Statement::Declaration(Box::new(Declaration::Extern(externs)))
    }

    /// Parse a block.
    fn parse_block(&mut self) -> Statement {
        self.expect(TokenType::OpenBrace);

        let mut body = vec![];

        while *self.peek() != TokenType::CloseBrace {
            body.push(self.parse_statement());
        }

        self.expect(TokenType::CloseBrace);

        Statement::Block(body)
    }

    /// Parse a statement.
    pub fn parse_statement(&mut self) -> Statement {
        let stat = match *self.peek() {
            TokenType::Keyword(Keyword::Return) => self.parse_return(),
            TokenType::Keyword(Keyword::If) => self.parse_if(),
            TokenType::Keyword(Keyword::Var) => self.parse_var_def(),
            TokenType::Keyword(Keyword::For) => self.parse_for(),
            TokenType::Keyword(Keyword::Fn) => self.parse_fn_def(),
            TokenType::Keyword(Keyword::Extern) => self.parse_extern(),
            TokenType::OpenBrace => self.parse_block(),
            _ => Statement::Expression(Box::new(self.parse_expression_statement())),
        };

        stat
    }

    fn parse_for(&mut self) -> Statement {
        self.expect(TokenType::Keyword(Keyword::For));

        self.expect(TokenType::OpenParen);
        self.expect(TokenType::CloseParen);

        let _body = self.parse_block();

        todo!()
    }

    /// Parse a variable definition.
    fn parse_var_def(&mut self) -> Statement {
        self.expect(TokenType::Keyword(Keyword::Var));

        let name = advance!(self => TokenType::Identifier);

        self.expect(TokenType::Colon);

        let typee = self.parse_type();

        self.expect(TokenType::Eq);

        let value = self.parse_expression();

        self.expect(TokenType::Semi);

        Statement::VarDef(name, typee, Box::new(value))
    }

    /// Parse if statement.
    fn parse_if(&mut self) -> Statement {
        self.expect(TokenType::Keyword(Keyword::If));

        self.expect(TokenType::OpenParen);

        let condition = self.parse_expression();

        self.expect(TokenType::CloseParen);

        let body = self.parse_block();
        let elif = {
            if *self.peek() == TokenType::Keyword(Keyword::Else) {
                Some(Box::new(self.parse_statement()))
            } else {
                None
            }
        };

        Statement::If(Box::new(condition), Box::new(body), elif)
    }

    /// Parse return statement.
    fn parse_return(&mut self) -> Statement {
        self.expect(TokenType::Keyword(Keyword::Return));

        let value = self.parse_expression();

        self.expect(TokenType::Semi);

        Statement::Return(Box::new(value))
    }

    /// Parse an expression statement.
    pub fn parse_expression_statement(&mut self) -> Expression {
        let expression = self.parse_expression();

        self.expect(TokenType::Semi);

        expression
    }

    /// Parse an expression.
    fn parse_expression(&mut self) -> Expression {
        self.parse_assignment()
    }

    /// Parse an identifier.
    fn parse_id(&mut self) -> Expression {
        let id = advance!(self => TokenType::Identifier);

        if *self.peek() == TokenType::OpenParen {
            let mut params = vec![];

            self.expect(TokenType::OpenParen);

            while *self.peek() != TokenType::CloseParen {
                params.push(self.parse_expression());

                if *self.peek() != TokenType::CloseParen {
                    self.expect(TokenType::Comma);
                }
            }

            self.expect(TokenType::CloseParen);

            Expression::FunctionCall(id, params)
        } else {
            Expression::VarRef(id)
        }
    }

    /// Parse a primary expression.
    fn parse_primary(&mut self) -> Expression {
        match self.peek().clone() {
            TokenType::Keyword(Keyword::True) => {
                advance!(self);
                Expression::Literal(Literal::Bool(true))
            }
            TokenType::Keyword(Keyword::False) => {
                advance!(self);
                Expression::Literal(Literal::Bool(false))
            }
            TokenType::Keyword(Keyword::Null) => {
                advance!(self);
                Expression::Literal(Literal::Null)
            }
            TokenType::Number(number) => {
                advance!(self);
                Expression::Literal(Literal::Number(number))
            }
            TokenType::Float(float) => {
                advance!(self);
                Expression::Literal(Literal::Float(float))
            }
            TokenType::String(string) => {
                advance!(self);
                Expression::Literal(Literal::String(string))
            }
            TokenType::Char(char) => {
                advance!(self);
                Expression::Literal(Literal::Char(char))
            }
            TokenType::Identifier(_) => self.parse_id(),
            TokenType::OpenParen => {
                advance!(self);

                let prime = self.parse_expression();
                advance!(self);

                Expression::Paren(Box::new(prime))
            }
            _ => panic!("Expected an expression, found `{:?}`", self.peek()),
        }
    }

    /// Parse a unary expression.
    fn parse_unary(&mut self) -> Expression {
        match self.peek() {
            TokenType::Minus => {
                advance!(self);

                let right = self.parse_unary();
                Expression::Unary(UnaryOp::Neg, Box::new(right))
            }
            TokenType::Bang => {
                advance!(self);

                let right = self.parse_unary();
                Expression::Unary(UnaryOp::Not, Box::new(right))
            }
            _ => self.parse_primary(),
        }
    }

    /// Parse assignment.
    fn parse_assignment(&mut self) -> Expression {
        let node = self.parse_or();

        if let TokenType::Eq = *self.peek() {
            advance!(self);

            let value = self.parse_expression();
            let var = match node {
                Expression::VarRef(var) => var,
                _ => panic!("Cannot assign value to `{:?}`", node),
            };

            return Expression::VarAssign(var, Box::new(value));
        }

        node
    }

    /// Parse or.
    fn parse_or(&mut self) -> Expression {
        let node = self.parse_and();

        match self.peek() {
            TokenType::PipePipe => {
                advance!(self);

                let rhs = self.parse_and();
                Expression::BinaryOp(Box::new(node), BinaryOp::Or, Box::new(rhs))
            }
            _ => node,
        }
    }

    /// Parse and.
    fn parse_and(&mut self) -> Expression {
        let node = self.parse_equality();

        match self.peek() {
            TokenType::AmpAmp => {
                advance!(self);

                let rhs = self.parse_equality();
                Expression::BinaryOp(Box::new(node), BinaryOp::And, Box::new(rhs))
            }
            _ => node,
        }
    }

    /// Parse equality.
    fn parse_equality(&mut self) -> Expression {
        let node = self.parse_comparison();

        match self.peek() {
            TokenType::EqEq => {
                advance!(self);

                let rhs = self.parse_comparison();
                Expression::BinaryOp(Box::new(node), BinaryOp::EqEq, Box::new(rhs))
            }
            _ => node,
        }
    }

    /// Parse comparison.
    fn parse_comparison(&mut self) -> Expression {
        let node = self.parse_term();

        match self.peek() {
            TokenType::Greater => {
                advance!(self);

                let rhs = self.parse_term();
                Expression::BinaryOp(Box::new(node), BinaryOp::Greater, Box::new(rhs))
            }
            TokenType::Lesser => {
                advance!(self);

                let rhs = self.parse_term();
                Expression::BinaryOp(Box::new(node), BinaryOp::Lesser, Box::new(rhs))
            }
            _ => node,
        }
    }

    /// Parse a term.
    fn parse_term(&mut self) -> Expression {
        let node = self.parse_factor();

        match self.peek() {
            TokenType::Plus => {
                advance!(self);

                let rhs = self.parse_factor();
                Expression::BinaryOp(Box::new(node), BinaryOp::Add, Box::new(rhs))
            }
            TokenType::Minus => {
                advance!(self);

                let rhs = self.parse_factor();
                Expression::BinaryOp(Box::new(node), BinaryOp::Subtract, Box::new(rhs))
            }
            _ => node,
        }
    }

    /// Parse a factor.
    fn parse_factor(&mut self) -> Expression {
        let node = self.parse_unary();

        match self.peek() {
            TokenType::Star => {
                advance!(self);

                let rhs = self.parse_unary();
                Expression::BinaryOp(Box::new(node), BinaryOp::Mul, Box::new(rhs))
            }
            TokenType::Slash => {
                advance!(self);

                let rhs = self.parse_unary();
                Expression::BinaryOp(Box::new(node), BinaryOp::Div, Box::new(rhs))
            }
            _ => node,
        }
    }

    fn expect(&mut self, token: TokenType) {
        if *self.peek() == token {
            advance!(self);
        } else {
            panic!("Expected {}", token)
        }
    }

    /// Peek the current token type.
    fn peek(&self) -> &TokenType {
        &self.tokens[self.index].kind
    }
}
