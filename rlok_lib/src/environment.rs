use super::error_handler::RuntimeError;
use super::lit::LitType;
use super::tokens::Token;
use color_eyre::eyre::{Report, Result};
use std::collections::HashMap;
use tracing::{span, trace, Level};

#[derive(Clone, Debug)]
pub struct Environment {
    values: HashMap<String, LitType>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new(enclosing: Option<Box<Environment>>) -> Self {
        trace!("Creating new environment");
        Environment {
            values: HashMap::new(),
            enclosing,
        }
    }

    pub fn define(&mut self, name: String, value: LitType) {
        let span_trace = span!(Level::TRACE, "env define");
        let _enter = span_trace.enter();
        self.values.insert(name, value);
        trace!(env = %self, "Environment Define");
    }

    pub fn get(&self, token: Token, span: String) -> Result<Option<LitType>> {
        let span_trace = span!(Level::TRACE, "env get");
        let _enter = span_trace.enter();
        if self.values.contains_key(&token.lexeme) {
            if let Some(val) = self.values.get(&token.lexeme) {
                trace!(get = %token);
                return Ok(Some(val.clone()));
            }
        }
        if let Some(ref enc) = self.enclosing {
            let span_trace = span!(Level::TRACE, "enclosing");
            let _enter = span_trace.enter();
            return Ok(enc.get(token.clone(), span)?);
        }
        Err(Report::new(RuntimeError::UndefinedVariable(
            token.lexeme,
            span,
        )))
    }

    pub fn assign(&mut self, name: Token, value: LitType, span: String) -> Result<()> {
        let span_trace = span!(Level::TRACE, "env assign");
        trace!(env = %self);
        let _enter = span_trace.enter();
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value.clone());
            trace!(name = %name.lexeme, value = %value);
            return Ok(());
        }
        if let Some(ref mut enc) = self.enclosing {
            let span_trace = span!(Level::TRACE, "enclosing");
            let _enter = span_trace.enter();
            enc.assign(name.clone(), value.clone(), span)?;
            return Ok(());
        }
        Err(Report::new(RuntimeError::UndefinedVariable(
            name.lexeme,
            span,
        )))
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let output = self
            .values
            .clone()
            .into_iter()
            .fold(String::new(), |acc, (k, v)| {
                if acc.len() > 0 {
                    format!("{}, {:?} = {:?}", acc, k, v)
                } else {
                    format!("{:?} = {:?}", k, v)
                }
            });
        write!(f, "{}", output)
    }
}
