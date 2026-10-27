use crate::diagnostics::Span;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeAnnotation {
    Int,
    Float,
    String,
    Bool,
    Unit,
    Inferred,
}

impl fmt::Display for TypeAnnotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeAnnotation::Int => write!(f, "Int"),
            TypeAnnotation::Float => write!(f, "Float"),
            TypeAnnotation::String => write!(f, "String"),
            TypeAnnotation::Bool => write!(f, "Bool"),
            TypeAnnotation::Unit => write!(f, "Unit"),
            TypeAnnotation::Inferred => write!(f, "Inferred"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Negate => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Subtract => write!(f, "-"),
            BinaryOp::Multiply => write!(f, "*"),
            BinaryOp::Divide => write!(f, "/"),
            BinaryOp::Modulo => write!(f, "%"),
            BinaryOp::Equal => write!(f, "=="),
            BinaryOp::NotEqual => write!(f, "!="),
            BinaryOp::Less => write!(f, "<"),
            BinaryOp::LessEqual => write!(f, "<="),
            BinaryOp::Greater => write!(f, ">"),
            BinaryOp::GreaterEqual => write!(f, ">="),
            BinaryOp::And => write!(f, "&&"),
            BinaryOp::Or => write!(f, "||"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(n) => write!(f, "{}", n),
            Literal::Float(n) => write!(f, "{}", n),
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal, Span),
    Variable(String, Span),
    Unary(UnaryOp, Box<Expr>, Span),
    Binary(Box<Expr>, BinaryOp, Box<Expr>, Span),
    Grouping(Box<Expr>, Span),
    Assign(String, Box<Expr>, Span),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, span) => *span,
            Expr::Variable(_, span) => *span,
            Expr::Unary(_, _, span) => *span,
            Expr::Binary(_, _, _, span) => *span,
            Expr::Grouping(_, span) => *span,
            Expr::Assign(_, _, span) => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    VarDecl {
        name: String,
        is_mutable: bool,
        type_annotation: Option<TypeAnnotation>,
        initializer: Expr,
        span: Span,
    },
    Expression(Expr, Span),
    Print(Expr, Span),
    Println(Expr, Span),
    Block(Vec<Stmt>, Span),
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::VarDecl { span, .. } => *span,
            Stmt::Expression(_, span) => *span,
            Stmt::Print(_, span) => *span,
            Stmt::Println(_, span) => *span,
            Stmt::Block(_, span) => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
