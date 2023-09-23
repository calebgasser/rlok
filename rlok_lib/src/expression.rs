use super::lit::LitType;
use super::tokens::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Option<LitType>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
    Assign {
        name: Token,
        value: Box<Expr>,
    },
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => write!(f, "( {} {} {} )", left, operator, right),
            Expr::Grouping { expression } => write!(f, "( {} )", expression),
            Expr::Literal { value } => {
                if let Some(val) = value {
                    write!(f, "{:?}", val)
                } else {
                    write!(f, "VALUE_MISSING")
                }
            }
            Expr::Unary { operator, right } => write!(f, "( {} {} )", operator, right),
            Expr::Variable { name } => write!(f, "{:?}", name),
            Expr::Assign { name, value } => write!(f, "( {} = {})", name, value),
        }
    }
}
