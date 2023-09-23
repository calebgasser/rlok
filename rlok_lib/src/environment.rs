use super::error_handler::RuntimeError;
use super::lit::LitType;
use super::tokens::Token;
use color_eyre::eyre::{Report, Result};
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, LitType>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
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
        Err(Report::new(RuntimeError::UndefinedVariable(token.lexeme)))
    }

    pub fn assign(&mut self, name: Token, value: LitType) -> Result<()> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme, value);
            return Ok(());
        }
        Err(Report::new(RuntimeError::UndefinedVariable(name.lexeme)))
    }
}
