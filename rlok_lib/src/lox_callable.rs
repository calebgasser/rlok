use super::environment::Environment;
use super::error_handler::RuntimeError;
use super::expression::Expr;
use super::interpreter::Interpreter;
use super::lit::LitType;
use super::statement::Statement;
use color_eyre::eyre::{Report, Result};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub enum LoxCallable {
    Function(LoxFunction),
    Clock(Clock),
}

pub trait Callable: std::fmt::Debug {
    fn new(callee: String, declaration: Option<Statement>) -> Self;
    fn callee(&self) -> String;
    fn call(&self, inter: &mut Interpreter, arguments: Vec<Box<Expr>>) -> Result<LitType>;
    fn arity(&self) -> usize;
    fn to_string(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct LoxFunction {
    declaration: Box<Option<Statement>>,
    callee: String,
}

#[derive(Debug, Clone)]
pub struct Clock {
    callee: String,
}

impl Callable for Clock {
    fn new(callee: String, _declaration: Option<Statement>) -> Self {
        Clock { callee }
    }
    fn callee(&self) -> String {
        self.callee.clone()
    }

    fn call(&self, _inter: &mut Interpreter, _arguments: Vec<Box<Expr>>) -> Result<LitType> {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(time) => Ok(LitType::Float(time.as_secs_f32())),
            Err(_) => Err(Report::new(RuntimeError::NativeFunctionError)),
        }
    }
    fn arity(&self) -> usize {
        0
    }
    fn to_string(&self) -> String {
        "<native fn>".into()
    }
}

impl Callable for LoxFunction {
    fn new(callee: String, declaration: Option<Statement>) -> Self {
        LoxFunction {
            callee,
            declaration: Box::new(declaration),
        }
    }
    fn callee(&self) -> String {
        self.callee.clone()
    }
    fn call(&self, inter: &mut Interpreter, arguments: Vec<Box<Expr>>) -> Result<LitType> {
        let mut environment = Environment::new(Some(Box::new(inter.globals.clone())));
        if let Some(declaration) = *self.declaration.clone() {
            if let Statement::Function { name, params, body } = declaration.clone() {
                for (index, param) in params.iter().enumerate() {
                    environment.define(
                        param.lexeme.clone(),
                        inter.evaluate_expr(*arguments[index].clone())?,
                    );
                }
                println!("Environment {:#?}", environment);
                inter.block_statement(body, environment)?;
            }

            if let Statement::Return { keyword: _, value } = declaration.clone() {
                println!("Returning: {}", value);
                return inter.evaluate_expr(value);
            }
        }
        println!("Returning nill");
        Ok(LitType::Nil)
    }
    fn arity(&self) -> usize {
        if let Some(dec) = *self.declaration.clone() {
            if let Statement::Function {
                name: _,
                params,
                body: _,
            } = dec
            {
                return params.len() as usize;
            }
        }
        0
    }

    fn to_string(&self) -> String {
        if let Some(dec) = *self.declaration.clone() {
            if let Statement::Function {
                name,
                params: _,
                body: _,
            } = dec
            {
                return format!("<fn {}>", name.lexeme);
            }
        }
        "<fn>".into()
    }
}
