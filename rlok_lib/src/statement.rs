use super::expression::Expr;
use super::span::Span;
use super::tokens::Token;

#[derive(Debug, Clone)]
pub enum Statement {
    Expression {
        span: Span,
        expression: Expr,
    },
    Print {
        span: Span,
        expression: Expr,
    },
    Var {
        span: Span,
        name: Token,
        expression: Option<Expr>,
    },
    Block {
        span: Span,
        statements: Vec<Box<Statement>>,
    },
    If {
        span: Span,
        condition: Expr,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        span: Span,
        condition: Expr,
        body: Box<Statement>,
    },
    Function {
        span: Span,
        name: Token,
        params: Vec<Token>,
        body: Vec<Box<Statement>>,
    },
    Return {
        span: Span,
        keyword: Token,
        value: Expr,
    },
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Statement::Expression {
                span: _,
                expression,
            } => write!(f, "{{ {:?} }}", expression),
            Statement::Print {
                span: _,
                expression,
            } => write!(f, "{{ print {:?} }}", expression),
            Statement::Var {
                span: _,
                name,
                expression,
            } => write!(f, "{{ {:?} = {:?} }}", name, expression),
            Statement::Block {
                span: _,
                statements,
            } => {
                let statements_out = statements.into_iter().fold(String::new(), |acc, stmt| {
                    if acc.len() > 0 {
                        format!("{:?}, {:?}", acc, stmt)
                    } else {
                        format!("{{ {:?} }}", stmt)
                    }
                });
                write!(f, "{:?}", statements_out)
            }
            Statement::If {
                span: _,
                condition,
                then_branch,
                else_branch,
            } => {
                if let Some(els) = else_branch {
                    write!(
                        f,
                        "{{ if {:?} then {:?} else {:?} }}",
                        condition, then_branch, els
                    )
                } else {
                    write!(f, "{{ if {:?} then {:?} }}", condition, then_branch)
                }
            }
            Statement::While {
                span: _,
                condition,
                body,
            } => {
                write!(f, "{{ while {:?} do {:?} }}", condition, body)
            }
            Statement::Function {
                span: _,
                name,
                params,
                body: _,
            } => {
                let params_output = params.into_iter().fold(String::new(), |acc, token| {
                    if acc.len() > 0 {
                        format!("{}, {:?}", acc, token)
                    } else {
                        format!("{:?}", token)
                    }
                });
                let body_output = params.into_iter().fold(String::new(), |acc, body| {
                    if acc.len() > 0 {
                        format!("{}, {:?}", acc, body)
                    } else {
                        format!("{:?}", body)
                    }
                });
                write!(
                    f,
                    "{{ function {:?} params {} body {} }}",
                    name, params_output, body_output
                )
            }
            Statement::Return {
                span: _,
                keyword,
                value,
            } => {
                write!(f, "{{ keyword {:?} value {:?} }}", keyword, value)
            }
        }
    }
}
