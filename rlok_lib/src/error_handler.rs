use super::expression::Expr;
use super::lit::LitType;
use super::statement::Statement;
use super::tokens::Token;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("[Error] [Parser] Connot have more than 255 arguments: {0}")]
    MaxArguments(Token),
    #[error("[Error] [Parser] While missing condition: {0}")]
    WhileMissingCondition(Token),
    #[error("[Error] [Parser] While missing body: {0}")]
    WhileMissingBody(Expr),
    #[error("[Error] [Parser] Logic 'and' missing right hand expression: {0}")]
    LogicAndMissingRight(Expr),
    #[error("[Error] [Parser] Logic 'or' missing right hand expression: {0}")]
    LogicOrMissingRight(Expr),
    #[error("[Error] [Parser] Missing if condition: {0}")]
    MissingIfCondition(Token),
    #[error("[Error] [Parser] Missing then branch: {0}")]
    MissingThenBranch(Expr),
    #[error("[Error] [Parser] Unexpected assignment target: {0}")]
    UnexpectedAssignmentTarget(Token),
    #[error("[Error] [Parser] Invalid assignment target: {0}")]
    InvalidAssignmentTarget(Token),
    #[error("[Error] [Parser] Expression missing expression: {0}")]
    ExpressionNoExpression(Token),
    #[error("[Error] [Parser] Print missing expression: {0}")]
    PrintNoExpression(Token),
    #[error("[Error] [Parser] Variable declaration error")]
    VarDeclartionError,
    #[error("[Error] [Parser] Variable expression missing on token: {0}")]
    VarMissingExpr(Token),
    #[error(
        "[Error] [Parser] [PrimaryTokenError] [line {line:?}] Error at {location:?}: {message:?}"
    )]
    PrimaryTokenError {
        line: i32,
        location: String,
        message: String,
    },
    #[error(
        "[Error] [Parser] [ConsumeTokenError] [line {line:?}] Error at {location:?}: {message:?}"
    )]
    ConsumeTokenError {
        line: i32,
        location: String,
        message: String,
    },
}

#[derive(Error, Debug)]
pub enum ScannerError {
    #[error("[Error] [Scanner] [UnexpectedToken] [line {line:?}] Error: {message:?}")]
    UnexpectedTokenError { line: i32, message: String },
    #[error("[Error] [Scanner] [StringError] [line {line:?}] Error: {message:?}")]
    StringError { line: i32, message: String },
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("return")]
    Return(LitType),
    #[error("[Error] [Runtime] [NativeFunctionError] Error calling native function")]
    NativeFunctionError,
    #[error("[Error] [Runtime] [IncorrectArgumentCount] Expected {0} arguments but got {1}.")]
    IncorrectArgumentCount(usize, usize),
    #[error("[Error] [Runtime] [NotCallable] Can only call functions and classes: {0}")]
    NotCallable(Token),
    #[error("[Error] [Runtime] [InvalidLiteral] {0}")]
    InvalidLiteral(Expr),
    #[error("[Error] [Runtime] [InvalidGrouping] {0}")]
    InvalidGrouping(Expr),
    #[error("[Error] [Runtime] [RightHandBoolOrNil] {0}")]
    RighthandBoolorNil(Expr),
    #[error("[Error] [Runtime] [UnaryExpects] Unary expect '!' or '1':  {0}")]
    UnaryExpects(Expr),
    #[error("[Error] [Runtime] [InvalidUnary] {0}")]
    InvalidUnary(Expr),
    #[error("[Error] [Runtime] [DivideByZero] {0}")]
    DivideByZero(Expr),
    #[error("[Error] [Runtime] [InvalidNumerical] Got expression {0} with token {1}")]
    InvalidNumerical(Expr, Token),
    #[error("[Error] [Runtime] [InvalidStringConcat] {0}")]
    InvalidStringConcat(Expr),
    #[error("[Error] [Runtime] [BinaryTypeMismatch] {0}")]
    BinaryTypeMismatch(Expr),
    #[error("[Error] [Runtime] [InvalidBinaryExpr] {0}")]
    InvalidBinaryExpr(Expr),
    #[error("[Error] [Runtime] Variable is undefined: {0} {1}")]
    UndefinedVariable(String, String),
    #[error("[Error] [Runtime] Expression is not a variable: {0}")]
    ExpressionNotVariable(Expr),
    #[error("[Error] [Runtime] Statement missing expression: {0}")]
    StatementMissingExpression(Statement),
    #[error("[Error] [Runtime] Statement not expected here: {0}")]
    UnexpectedStatement(Statement),
    #[error("[Error] [Runtime] Invalid assignment target: {0} with {1}")]
    InvalidAssignmentTarget(Token, Expr),
}
