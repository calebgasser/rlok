use super::error_handler::RuntimeError;
use super::expression::Expr;
use super::interpreter::Interpreter;
use super::lit::LitType;
use color_eyre::eyre::{Report, Result};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub enum LoxCallable {
    Function(LoxFunction),
    Clock(Clock),
}

pub trait Callable: std::fmt::Debug {
    fn new(callee: String) -> Self;
    fn callee(&self) -> String;
    fn call(&self, inter: Interpreter, arguments: Vec<Expr>) -> Result<LitType>;
    fn arity(&self) -> usize;
    fn to_string(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct LoxFunction {
    callee: String,
}

#[derive(Debug, Clone)]
pub struct Clock {
    callee: String,
}

impl Callable for Clock {
    fn new(callee: String) -> Self {
        Clock { callee }
    }
    fn callee(&self) -> String {
        self.callee
    }
    fn call(&self, inter: Interpreter, arguments: Vec<Expr>) -> Result<LitType> {
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
    fn new(callee: String) -> Self {
        LoxFunction { callee }
    }
    fn callee(&self) -> String {
        self.callee
    }
    fn call(&self, inter: Interpreter, arguments: Vec<Expr>) -> Result<LitType> {
        Ok(LitType::Nil)
    }
    fn arity(&self) -> usize {
        0
    }

    fn to_string(&self) -> String {
        "<fn>".into()
    }
}
