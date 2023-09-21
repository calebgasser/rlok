use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::io::Write;

pub struct ErrorHandler {
    has_error: bool,
}

impl ErrorHandler {
    pub fn build() -> Self {
        ErrorHandler { has_error: false }
    }

    pub fn error(&mut self, line: i32, message: &str) {
        self.has_error = true;
        self.report(line, String::from(""), message);
    }

    fn report(&self, line: i32, location: String, message: &str) {
        println!("[line {line}] Error {location}: {message}");
    }
}

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

pub struct Scanner {
    error_handler: ErrorHandler,
    source: String,
    start: i32,
    current: i32,
    line: i32,
    tokens: Vec<Token>,
    keywords: HashMap<String, TokenType>,
}

impl Scanner {
    pub fn build(source: String) -> Self {
        let error_handler = ErrorHandler::build();
        let mut keywords = HashMap::new();
        keywords.insert("and".into(), TokenType::AND);
        keywords.insert("class".into(), TokenType::CLASS);
        keywords.insert("else".into(), TokenType::ELSE);
        keywords.insert("false".into(), TokenType::FALSE);
        keywords.insert("for".into(), TokenType::FOR);
        keywords.insert("fun".into(), TokenType::FUN);
        keywords.insert("if".into(), TokenType::IF);
        keywords.insert("nil".into(), TokenType::NIL);
        keywords.insert("or".into(), TokenType::OR);
        keywords.insert("print".into(), TokenType::PRINT);
        keywords.insert("return".into(), TokenType::RETURN);
        keywords.insert("super".into(), TokenType::SUPER);
        keywords.insert("this".into(), TokenType::THIS);
        keywords.insert("true".into(), TokenType::TRUE);
        keywords.insert("var".into(), TokenType::VAR);
        keywords.insert("while".into(), TokenType::WHILE);
        Scanner {
            error_handler,
            source,
            start: 0,
            current: 0,
            line: 1,
            tokens: Vec::new(),
            keywords,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens
            .push(Token::new(TokenType::EOF, "".into(), None, self.line));
        self.tokens.clone()
    }

    fn is_end(&self) -> bool {
        self.current >= self.source.len() as i32
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current as usize).unwrap();
        self.current += 1;
        c
    }

    fn add_token(&mut self, ty: TokenType) {
        let text = &self.source[self.start as usize..self.current as usize];
        self.tokens
            .push(Token::new(ty, text.into(), None, self.line));
    }

    fn add_token_val(&mut self, ty: TokenType, value: &str) {
        let text = &self.source[self.start as usize..self.current as usize];
        self.tokens.push(Token::new(
            ty,
            text.into(),
            Some(value.to_string()),
            self.line,
        ));
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_end() {
            return false;
        }
        if self.source.chars().nth(self.current as usize).unwrap() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current as usize).unwrap()
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() as i32 {
            return '\0';
        }
        self.source
            .chars()
            .nth((self.current + 1) as usize)
            .unwrap()
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_end() {
            self.error_handler.error(self.line, "Unterminated string.");
        } else {
            self.advance();

            let value = &self.source[(self.start + 1) as usize..(self.current - 1) as usize];
            self.add_token_val(TokenType::StringLit, &value.to_string());
        }
    }

    fn number(&mut self) {
        while Self::is_digit(self.peek()) {
            self.advance();
        }
        if self.peek() == '.' && Self::is_digit(self.peek_next()) {
            self.advance();
            while Self::is_digit(self.peek()) {
                self.advance();
            }
        }
        let value = &self.source[self.start as usize..self.current as usize];
        self.add_token_val(TokenType::NumberLit, &value.to_string());
    }

    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn identifier(&mut self) {
        while Self::is_alpha_numeric(self.peek()) {
            self.advance();
        }
        let value = &self.source[self.start as usize..self.current as usize];
        let ty = self.keywords.get(value);
        if let Some(t) = ty {
            self.add_token(t.clone());
        } else {
            self.add_token(TokenType::Ident);
        }
    }

    fn is_alpha(c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }

    fn is_alpha_numeric(c: char) -> bool {
        Self::is_alpha(c) || Self::is_digit(c)
    }

    fn scan_token(&mut self) {
        let c: char = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            '"' => self.string(),
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            _ => {
                if Self::is_digit(c) {
                    self.number();
                } else if Self::is_alpha(c) {
                    self.identifier();
                } else {
                    self.error_handler.error(self.line, "Unexpected token");
                }
            }
        }
    }
}

pub struct Interpreter;

impl Interpreter {
    pub fn build() -> Self {
        Interpreter
    }

    pub fn start(&self, args: Vec<String>) {
        if args.len() == 2 {
            self.run_file(&args[1]);
        } else {
            self.run_prompt();
        }
    }

    fn run(&self, contents: String) {
        let mut scanner = Scanner::build(contents);
        let tokens = scanner.scan_tokens();
        println!("Tokens: {:?}", tokens);
    }

    fn run_file(&self, file: &str) {
        let contents = fs::read_to_string(file).expect("Unable to read file.");
        self.run(contents);
    }

    fn run_prompt(&self) {
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let mut buffer = String::new();
            io::stdin()
                .read_line(&mut buffer)
                .expect("Unable to read line");
            if buffer.len() <= 1 {
                break;
            } else {
                self.run(buffer);
            }
        }
    }
}

fn main() {
    Interpreter::build().start(env::args().collect());
}
