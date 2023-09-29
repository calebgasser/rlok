use super::lit::LitType;
use super::span::Span;
use super::tokens::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Binary {
        span: Span,
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        span: Span,
        expression: Box<Expr>,
    },
    Literal {
        span: Span,
        value: Option<LitType>,
    },
    Unary {
        span: Span,
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        span: Span,
        name: Token,
    },
    Assign {
        span: Span,
        name: Token,
        value: Box<Expr>,
    },
    Logcial {
        span: Span,
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        span: Span,
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Box<Expr>>,
    },
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Binary {
                span: _,
                left,
                operator,
                right,
            } => write!(f, "{:?} {:?} {:?}", left, operator, right),
            Expr::Grouping {
                span: _,
                expression,
            } => write!(f, "{:?}", expression),
            Expr::Literal { span: _, value } => {
                if let Some(val) = value {
                    write!(f, "{:?}", val)
                } else {
                    write!(f, "VALUE_MISSING")
                }
            }
            Expr::Unary {
                span: _,
                operator,
                right,
            } => write!(f, "{:?}{:?} ", operator, right),
            Expr::Variable { span: _, name } => write!(f, "{:?}", name),
            Expr::Assign {
                span: _,
                name,
                value,
            } => write!(f, "{:?} = {:?}", name, value),
            Expr::Logcial {
                span: _,
                left,
                operator,
                right,
            } => write!(f, "{:?} {:?} {:?}", left, operator, right),
            Expr::Call {
                span: _,
                callee,
                paren: _,
                arguments,
            } => {
                let arguments_out = arguments.into_iter().fold(String::new(), |acc, arg| {
                    if acc.len() > 0 {
                        format!("{:?}, {:?}", acc, arg)
                    } else {
                        format!("{:?}", arg)
                    }
                });
                write!(f, "{:?}({:?})", callee, arguments_out)
            }
        }
    }
}
