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
    If {
        condition: Expr,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        condition: Expr,
        body: Box<Statement>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Box<Statement>>,
    },
    Return {
        keyword: Token,
        value: Expr,
    },
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Statement::Expression { expression } => write!(f, "{{ {} }}", expression),
            Statement::Print { expression } => write!(f, "{{ print {} }}", expression),
            Statement::Var { name, expression } => write!(f, "{{ {} = {:?} }}", name, expression),
            Statement::Block { statements } => write!(f, "{{ {:?} }}", statements),
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if let Some(els) = else_branch {
                    write!(
                        f,
                        "{{ if {} then {} else {} }}",
                        condition, then_branch, els
                    )
                } else {
                    write!(f, "{{ if {} then {} }}", condition, then_branch)
                }
            }
            Statement::While { condition, body } => {
                write!(f, "{{ while {} do {} }}", condition, body)
            }
            Statement::Function { name, params, body } => {
                write!(
                    f,
                    "{{ function {} params {:?} body {:?} }}",
                    name, params, body
                )
            }
            Statement::Return { keyword, value } => {
                write!(f, "{{ keyword {} value {} }}", keyword, value)
            }
        }
    }
}
