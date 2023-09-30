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
        self.values.insert(name, value);
        trace!(env = %self, "Environment Define");
    }

    pub fn get(&self, token: Token, span: String) -> Result<Option<LitType>> {
        if self.values.contains_key(&token.lexeme) {
            if let Some(val) = self.values.get(&token.lexeme) {
                trace!(get = %token, "Environment Get");
                return Ok(Some(val.clone()));
            }
        }
        if let Some(ref enc) = self.enclosing {
            return Ok(enc.get(token.clone(), span)?);
        }
        Err(Report::new(RuntimeError::UndefinedVariable(
            token.lexeme,
            span,
        )))
    }

    pub fn assign(&mut self, name: Token, value: LitType, span: String) -> Result<()> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value.clone());
            trace!(name = %name.lexeme, value = %value, "Environment Assign");
            return Ok(());
        }
        if let Some(ref mut enc) = self.enclosing {
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
        fn get_enclosing(env: &Environment) -> String {
            let mut values = String::new();
            if env.values.len() > 0 {
                values = env
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
            }
            if let Some(ref enc) = env.enclosing {
                let last_val = get_enclosing(enc);
                if values.len() > 0 {
                    return format!("{}, {}", values, last_val);
                } else {
                    return format!("{}", last_val);
                }
            } else {
                return values;
            }
        }
        write!(f, "{}", get_enclosing(self))
    }
}
