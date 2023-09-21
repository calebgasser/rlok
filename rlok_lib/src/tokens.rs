#[derive(Debug, Clone)]
pub enum TokenType {
    // Signel-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Ident,
    StringLit,
    NumberLit,

    // Keywords
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    ty: TokenType,
    lexeme: String,
    literal: Option<String>,
    line: i32,
}

impl Token {
    pub fn new(ty: TokenType, lexeme: String, literal: Option<String>, line: i32) -> Self {
        Token {
            ty,
            lexeme,
            literal,
            line,
        }
    }
}
