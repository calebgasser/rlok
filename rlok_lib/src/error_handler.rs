use thiserror::Error;

#[derive(Error, Debug)]
pub enum BaseError {
    #[error("[line {line:?}] Error: {message:?}")]
    LineError { line: i32, message: String },
    #[error("[line {line:?}] Error  at {location:?}: {message:?}")]
    TokenError {
        line: i32,
        location: String,
        message: String,
    },
}
