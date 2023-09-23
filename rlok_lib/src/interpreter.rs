use super::environment::Environment;
use super::error_handler::RuntimeError;
use super::expression::Expr;
use super::lit::LitType;
use super::parser::Parser;
use super::scanner::Scanner;
use super::statement::Statement;
use super::tokens::TokenType;
use color_eyre::eyre::{Report, Result};
use std::fs;
use std::io;
use std::io::Write;

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn build() -> Self {
        Interpreter {
            environment: Environment::new(None),
        }
    }

    pub fn start(&mut self, args: Vec<String>) -> Result<()> {
        if args.len() == 2 {
            self.run_file(&args[1])?;
        } else {
            self.run_prompt()?;
        }
        Ok(())
    }

    fn run(&mut self, contents: String) -> Result<()> {
        let mut scanner = Scanner::build(contents);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(tokens.clone())?;
        match parser.parse() {
            Ok(ast) => {
                if let Some(ast) = ast {
                    for stmt in &ast {
                        match self.evaluate_statement(stmt.clone()) {
                            Ok(output) => {
                                if let Some(out) = output {
                                    Self::print_lit(out);
                                }
                            }
                            Err(e) => {
                                eprintln!("{}", e);
                            }
                        }
                    }
                } else {
                    eprintln!("Failed to generate AST")
                }
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
        Ok(())
    }

    fn run_file(&mut self, file: &str) -> Result<()> {
        let contents = fs::read_to_string(file)?;
        self.run(contents)?;
        Ok(())
    }

    fn run_prompt(&mut self) -> Result<()> {
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let mut buffer = String::new();
            io::stdin()
                .read_line(&mut buffer)
                .expect("Unable to read line");
            if buffer.len() <= 1 {
                break;
            } else {
                match self.run(buffer) {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }

    fn literal_expr(&self, expr: Expr) -> Result<LitType> {
        if let Expr::Literal { value } = expr.clone() {
            if let Some(val) = value {
                return Ok(val);
            }
        }
        Err(Report::new(RuntimeError::InvalidLiteral(expr)))
    }

    fn grouping_expr(&mut self, expr: Expr) -> Result<LitType> {
        if let Expr::Grouping { expression } = expr {
            return self.evaluate_expr(*expression);
        }
        Err(Report::new(RuntimeError::InvalidGrouping(expr)))
    }

    fn unary_expr(&mut self, expr: Expr) -> Result<LitType> {
        if let Expr::Unary { operator, right } = expr.clone() {
            let right = self.evaluate_expr(*right)?;
            match operator.ty {
                TokenType::Minus => {
                    if let LitType::Float(f) = right {
                        return Ok(LitType::Float(-f));
                    }
                }
                TokenType::Bang => match right {
                    LitType::Bool(b) => return Ok(LitType::Bool(!b)),
                    LitType::Nil => return Ok(LitType::Bool(true)),
                    _ => return Err(Report::new(RuntimeError::RighthandBoolorNil(expr))),
                },
                _ => return Err(Report::new(RuntimeError::UnaryExpects(expr))),
            }
        }
        Err(Report::new(RuntimeError::InvalidUnary(expr)))
    }

    fn binary_expr(&mut self, expr: Expr) -> Result<LitType> {
        if let Expr::Binary {
            left,
            operator,
            right,
        } = expr.clone()
        {
            let left = self.evaluate_expr(*left)?;
            let right = self.evaluate_expr(*right)?;
            if let LitType::Float(r) = right {
                if let LitType::Float(l) = left {
                    match operator.ty {
                        TokenType::Plus => return Ok(LitType::Float(l + r)),
                        TokenType::Minus => return Ok(LitType::Float(l - r)),
                        TokenType::Slash => {
                            if r == 0.0 {
                                return Err(Report::new(RuntimeError::DivideByZero(expr)));
                            } else {
                                return Ok(LitType::Float(l / r));
                            }
                        }
                        TokenType::Star => return Ok(LitType::Float(l * r)),
                        _ => return Err(Report::new(RuntimeError::InvalidNumerical(expr))),
                    }
                }
            }
            if let LitType::Str(r) = right {
                if let LitType::Str(l) = left {
                    match operator.ty {
                        TokenType::Plus => return Ok(LitType::Str(format!("{}{}", l, r))),
                        _ => return Err(Report::new(RuntimeError::InvalidStringConcat(expr))),
                    }
                }
            }
            return Err(Report::new(RuntimeError::BinaryTypeMismatch(expr)));
        }
        Err(Report::new(RuntimeError::InvalidBinaryExpr(expr)))
    }

    fn var_expr(&self, expr: Expr) -> Result<LitType> {
        match expr {
            Expr::Variable { name } => {
                if let Some(val) = self.environment.get(name.clone())? {
                    return Ok(val);
                }
                return Err(Report::new(RuntimeError::UndefinedVariable(name.lexeme)));
            }
            _ => Err(Report::new(RuntimeError::ExpressionNotVariable(expr))),
        }
    }

    fn print_lit(lit: LitType) {
        match lit {
            LitType::Float(flt) => println!("{}", flt),
            LitType::Str(str) => println!("{}", str),
            LitType::Bool(bl) => println!("{}", bl),
            LitType::Nil => println!("nil"),
        }
    }

    fn var_statement(&mut self, stmt: Statement) -> Result<()> {
        if let Statement::Var { name, expression } = stmt.clone() {
            if let Some(expr) = expression {
                let value = self.evaluate_expr(expr)?;
                self.environment.define(name.lexeme, value);
                return Ok(());
            }
            return Err(Report::new(RuntimeError::StatementMissingExpression(stmt)));
        }
        Err(Report::new(RuntimeError::UnexpectedStatement(stmt)))
    }

    fn execute_block(
        &mut self,
        statements: Vec<Box<Statement>>,
        environment: Environment,
    ) -> Result<()> {
        let previous = self.environment.clone();
        self.environment = environment;
        for stmt in statements {
            self.evaluate_statement(*stmt)?;
        }
        self.environment = previous;
        Ok(())
    }

    fn evaluate_statement(&mut self, stmt: Statement) -> Result<Option<LitType>> {
        match stmt.clone() {
            Statement::Print { expression } => {
                let value = self.evaluate_expr(expression)?;
                Self::print_lit(value);
                Ok(None)
            }
            Statement::Expression { expression } => {
                return Ok(Some(self.evaluate_expr(expression)?));
            }
            Statement::Var {
                name: _,
                expression: _,
            } => {
                self.var_statement(stmt)?;
                return Ok(None);
            }
            Statement::Block { statements } => {
                self.execute_block(
                    statements,
                    Environment::new(Some(Box::new(self.environment.clone()))),
                )?;
                Ok(None)
            }
        }
    }

    fn evaluate_expr(&mut self, expr: Expr) -> Result<LitType> {
        match &expr {
            Expr::Binary {
                left: _,
                operator: _,
                right: _,
            } => Ok(self.binary_expr(expr)?),
            Expr::Grouping { expression: _ } => Ok(self.grouping_expr(expr)?),
            Expr::Unary {
                operator: _,
                right: _,
            } => Ok(self.unary_expr(expr)?),
            Expr::Literal { value: _ } => Ok(self.literal_expr(expr)?),
            Expr::Variable { name: _ } => Ok(self.var_expr(expr)?),
            Expr::Assign { name, value } => {
                if let Expr::Literal { ref value } = **value {
                    if let Some(val) = value {
                        self.environment.assign(name.clone(), val.clone())?;
                        return Ok(val.clone());
                    }
                }
                Err(Report::new(RuntimeError::InvalidAssignmentTarget(
                    name.clone(),
                )))
            }
        }
    }
}
