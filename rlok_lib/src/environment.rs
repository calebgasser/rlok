use super::error_handler::RuntimeError;
use super::lit::LitType;
use super::tokens::Token;
use color_eyre::eyre::{Report, Result};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Environment {
    values: HashMap<String, LitType>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new(enclosing: Option<Box<Environment>>) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing,
        }
    }

    pub fn define(&mut self, name: String, value: LitType) {
        self.values.insert(name, value);
    }

    pub fn get(&self, token: Token) -> Result<Option<LitType>> {
        if self.values.contains_key(&token.lexeme) {
            if let Some(val) = self.values.get(&token.lexeme) {
                return Ok(Some(val.clone()));
            }
        }
        if let Some(ref enc) = self.enclosing {
            return Ok(enc.get(token.clone())?);
        }
        Err(Report::new(RuntimeError::UndefinedVariable(token.lexeme)))
    }

    pub fn assign(&mut self, name: Token, value: LitType) -> Result<()> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme, value);
            return Ok(());
        }
        if let Some(ref mut enc) = self.enclosing {
            enc.assign(name.clone(), value.clone())?;
            return Ok(());
        }
        Err(Report::new(RuntimeError::UndefinedVariable(name.lexeme)))
    }
}
