use super::environment::Environment;
use super::error_handler::RuntimeError;
use super::expression::Expr;
use super::interpreter::Interpreter;
use super::lit::LitType;
use super::statement::Statement;
use color_eyre::eyre::{Report, Result};
use std::time::SystemTime;
use tracing::trace;

#[derive(Debug, Clone)]
pub enum LoxCallable {
    Function(LoxFunction),
    Clock(Clock),
}

impl std::fmt::Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LoxCallable::Function(func) => {
                write!(f, "{:?}", func)
            }
            LoxCallable::Clock(clock) => {
                write!(f, "{:?}", clock)
            }
        }
    }
}
pub trait Callable: std::fmt::Debug + std::fmt::Display {
    fn new(callee: String, declaration: Option<Statement>) -> Self;
    fn callee(&self) -> String;
    fn call(&self, inter: &mut Interpreter, arguments: Vec<Box<Expr>>) -> Result<LitType>;
    fn arity(&self) -> usize;
    fn as_string(&self) -> String;
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

impl std::fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<fun {}>", self.callee)
    }
}

impl std::fmt::Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<fun {}>", self.callee)
    }
}

impl Callable for Clock {
    fn new(callee: String, _declaration: Option<Statement>) -> Self {
        trace!(callee, "Creating function");
        Clock { callee }
    }

    fn callee(&self) -> String {
        self.callee.clone()
    }

    fn call(&self, _inter: &mut Interpreter, _arguments: Vec<Box<Expr>>) -> Result<LitType> {
        trace!("Callling clock function");
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(time) => Ok(LitType::Float(time.as_secs_f32())),
            Err(_) => Err(Report::new(RuntimeError::NativeFunctionError)),
        }
    }
    fn arity(&self) -> usize {
        0
    }
    fn as_string(&self) -> String {
        "<native fn>".into()
    }
}

impl Callable for LoxFunction {
    fn new(callee: String, declaration: Option<Statement>) -> Self {
        trace!(callee, "Creating function");
        LoxFunction {
            callee,
            declaration: Box::new(declaration),
        }
    }

    fn callee(&self) -> String {
        self.callee.clone()
    }

    fn call(&self, inter: &mut Interpreter, arguments: Vec<Box<Expr>>) -> Result<LitType> {
        let mut environment = Environment::new(Some(Box::new(inter.environment.clone())));
        if let Some(declaration) = *self.declaration.clone() {
            if let Statement::Function {
                span: _,
                name,
                params,
                body,
            } = declaration.clone()
            {
                trace!(name = %name, "Called function");
                for (index, param) in params.iter().enumerate() {
                    trace!(param = %param, index, "parameter");
                    environment.define(
                        param.lexeme.clone(),
                        inter.evaluate_expr(*arguments[index].clone())?,
                    );
                }
                if let Some(result) = inter.block_statement(body, environment)? {
                    return Ok(result);
                }
            }

            if let Statement::Return {
                span: _,
                keyword: _,
                value,
            } = declaration.clone()
            {
                return inter.evaluate_expr(value);
            }
        }
        println!("Returning nill");
        Ok(LitType::Nil)
    }

    fn arity(&self) -> usize {
        if let Some(dec) = *self.declaration.clone() {
            if let Statement::Function {
                span: _,
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

    fn as_string(&self) -> String {
        if let Some(dec) = *self.declaration.clone() {
            if let Statement::Function {
                span: _,
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
