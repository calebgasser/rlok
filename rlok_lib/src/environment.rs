use super::error_handler::RuntimeError;
use super::lit::LitType;
use super::tokens::Token;
use color_eyre::eyre::{Report, Result};
use std::collections::HashMap;
use tracing::trace;

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
        self.values.insert(name, value);
        trace!(env = %self, "Environment Define");
    }

    pub fn get(&self, token: Token) -> Result<Option<LitType>> {
        if self.values.contains_key(&token.lexeme) {
            if let Some(val) = self.values.get(&token.lexeme) {
                trace!(get = %token, val = %val.clone(), env = %self,"Environment Get");
                return Ok(Some(val.clone()));
            }
        } else if let Some(ref enc) = self.enclosing {
            trace!(get = %token, enclosing = %enc, "Environment Enclosing Get");
            return Ok(enc.get(token.clone())?);
        }
        Err(Report::new(RuntimeError::UndefinedVariable(token.lexeme)))
    }

    pub fn assign(&mut self, name: Token, value: LitType) -> Result<()> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value.clone());
            trace!(env = %self, "Environment Assign");
            trace!(name = %name.lexeme, value = %value);
            return Ok(());
        } else if let Some(ref mut enc) = self.enclosing {
            enc.assign(name.clone(), value.clone())?;
            trace!(encolsing = %enc, "Environment Enclosing Assign");
            trace!(name = %name, value = %value);
            return Ok(());
        }
        Err(Report::new(RuntimeError::UndefinedVariable(name.lexeme)))
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let output = self
            .values
            .clone()
            .into_iter()
            .fold(String::from(""), |acc, (k, v)| {
                if acc.len() > 0 {
                    format!("{}, {:?}={:?}", acc, k, v)
                } else {
                    format!("{}={:?}", k, v)
                }
            });
        write!(f, "{}", output)
    }
}
