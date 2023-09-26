use super::lox_callable::LoxCallable;

#[derive(Debug, Clone)]
pub enum LitType {
    Float(f32),
    Str(String),
    Bool(bool),
    Callable(LoxCallable),
    Nil,
}

impl std::fmt::Display for LitType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LitType::Float(flt) => write!(f, "{}", flt),
            LitType::Str(str) => write!(f, "{}", str),
            LitType::Bool(bl) => write!(f, "{}", bl),
            LitType::Callable(call) => write!(f, "{:?}", call),
            LitType::Nil => write!(f, "nil"),
        }
    }
}
