use super::expression::Expr;
use super::tokens::Token;

#[derive(Debug, Clone)]
pub enum Statement {
    Expression {
        expression: Expr,
    },
    Print {
        expression: Expr,
    },
    Var {
        name: Token,
        expression: Option<Expr>,
    },
    Block {
        statements: Vec<Box<Statement>>,
    },
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Statement::Expression { expression } => write!(f, "( {} )", expression),
            Statement::Print { expression } => write!(f, "{}", expression),
            Statement::Var { name, expression } => write!(f, "( {} = {:?} )", name, expression),
            Statement::Block { statements } => write!(f, "{{ {:?} }}", statements),
        }
    }
}
