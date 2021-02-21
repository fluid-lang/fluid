//! This file contains all of the AST interfaces.

/// An expression.
#[derive(Debug)]
pub enum Expression {
    /// A variable reference.
    VarRef(String),
    /// A variable assign.
    VarAssign(String, Box<Expression>),
    /// A function call.
    FunctionCall(String, Vec<Expression>),
    /// A binary operator.
    BinaryOp(Box<Expression>, BinaryOp, Box<Expression>),
    /// A literal expression.
    Literal(Literal),
    /// An unary expression.
    Unary(UnaryOp, Box<Expression>),
    /// A paren expression.
    Paren(Box<Expression>),
}

/// An unary operator.
#[derive(Debug)]
pub enum UnaryOp {
    /// `-`
    Neg,
    /// `!`
    Not,
}

/// A binary operator.
#[derive(Debug)]
pub enum BinaryOp {
    /// `+`
    Add,
    /// `-`
    Subtract,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `<`
    Lesser,
    /// `>`
    Greater,
    /// `==`
    EqEq,
    /// `&&`
    And,
    /// `||`
    Or,
}

/// A literal.
#[derive(Debug)]
pub enum Literal {
    /// A boolean literal.
    Bool(bool),
    /// A number literal.
    Number(u64),
    /// A floating point.
    Float(f64),
    /// A string literal.
    String(String),
    /// A character.
    Char(char),
    /// Null.
    Null,
}

/// A statement.
#[derive(Debug)]
pub enum Statement {
    /// An expression statement.
    Expression(Box<Expression>),
    /// Return statement.
    Return(Box<Expression>),
    /// If statement.
    If(Box<Expression>, Box<Statement>, Option<Box<Statement>>),
    /// For statement.
    For(),
    /// A block statement.
    Block(Vec<Statement>),
    /// A declaration statement.
    Declaration(Box<Declaration>),
}

/// A declaration.
#[derive(Debug)]
pub enum Declaration {
    /// A function declaration.
    Function(Function),
    /// An external declaration.
    Extern(Vec<Prototype>),
    /// A variable declaration.
    VarDef(String, Type, Box<Expression>),
}

/// A function
#[derive(Debug)]
pub struct Function {
    /// The function prototype.
    pub prototype: Prototype,
    /// The function body.
    pub body: Statement,
}

/// Function's prototype.
#[derive(Debug)]
pub struct Prototype {
    /// The function name.
    pub name: String,
    /// The function args.
    pub args: Vec<Arg>,
    /// The function return type.
    pub return_type: Type,
}

/// A function argument
#[derive(Debug)]
pub struct Arg {
    /// Name of the argument.
    pub name: String,
    /// Type of the argument.
    pub typee: Type,
}

/// A type.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Type {
    /// void
    Void,
    /// number
    Number,
    /// float
    Float,
    /// string
    String,
    /// bool
    Bool,
}

impl Default for Type {
    /// Return the default type. `void`
    fn default() -> Self {
        Self::Void
    }
}
