use super::error_handler::BaseError;
use super::tokens::{Token, TokenType};
use color_eyre::eyre::{Report, Result};

// expression     → literal
//                | unary
//                | binary
//                | grouping ;
// literal        → NUMBER | STRING | "true" | "false" | "nil" ;
// grouping       → "(" expression ")" ;
// unary          → ( "-" | "!" ) expression ;
// binary         → expression operator expression ;
// operator       → "==" | "!=" | "<" | "<=" | ">" | ">="
//                | "+"  | "-"  | "*" | "/" ;
// ---------------------------------------------------------------
// expression     → comma ;
// comma          → equality ( "," equality )*;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" ;

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Option<String>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => write!(f, "Binary( {} {} {} )", left, operator, right),
            Expr::Grouping { expression } => write!(f, "Grouping( {} )", expression),
            Expr::Literal { value } => {
                if let Some(val) = value {
                    write!(f, "{:?}", val)
                } else {
                    write!(f, "VALUE_MISSING")
                }
            }
            Expr::Unary { operator, right } => write!(f, "Unary( {} {} )", operator, right),
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: i32,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Result<Self> {
        Ok(Parser { tokens, current: 0 })
    }

    fn peek(&self) -> Token {
        self.tokens[self.current as usize].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[(self.current - 1) as usize].clone()
    }

    fn is_end(&self) -> bool {
        matches!(self.peek().ty, TokenType::EOF)
    }

    fn consume(&mut self, ty: TokenType, error_message: &str) -> Result<()> {
        if self.check(ty) {
            self.advance();
            Ok(())
        } else {
            let token = self.peek();
            Err(Report::new(BaseError::TokenError {
                line: token.line,
                location: token.lexeme,
                message: error_message.into(),
            }))
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn check(&self, ty: TokenType) -> bool {
        if self.is_end() {
            false
        } else {
            if self.peek().ty == ty {
                true
            } else {
                false
            }
        }
    }

    fn match_type(&mut self, types: Vec<TokenType>) -> bool {
        for ty in types {
            if self.check(ty) {
                self.advance();
                return true;
            }
        }
        false
    }

    pub fn parse(&mut self) -> Result<Option<Expr>> {
        if let Some(expr) = self.expression()? {
            return Ok(Some(expr));
        }
        Ok(None)
    }
    // expression     → comma ;
    fn expression(&mut self) -> Result<Option<Expr>> {
        if let Some(expr) = self.comma()? {
            return Ok(Some(expr));
        }
        Ok(None)
    }

    // comma          → equality ( "," equality )*;
    fn comma(&mut self) -> Result<Option<Expr>> {
        if let Some(mut expr) = self.equality()? {
            while self.match_type(vec![TokenType::Comma]) {
                let operator = self.previous();
                if let Some(right) = self.equality()? {
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    }
                }
            }
            return Ok(Some(expr));
        }
        Ok(None)
    }

    // equality       → comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Option<Expr>> {
        if let Some(mut expr) = self.comparison()? {
            while self.match_type(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
                let operator = self.previous();
                if let Some(right) = self.comparison()? {
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    };
                }
            }
            return Ok(Some(expr));
        }
        Ok(None)
    }

    // comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Result<Option<Expr>> {
        if let Some(mut expr) = self.term()? {
            while self.match_type(vec![
                TokenType::Greater,
                TokenType::LessEqual,
                TokenType::Less,
                TokenType::LessEqual,
            ]) {
                let operator = self.previous();
                if let Some(right) = self.term()? {
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    };
                }
            }
            return Ok(Some(expr));
        }
        Ok(None)
    }
    // term           → factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Result<Option<Expr>> {
        if let Some(mut expr) = self.factor()? {
            while self.match_type(vec![TokenType::Minus, TokenType::Plus]) {
                let operator = self.previous();
                if let Some(right) = self.factor()? {
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    }
                }
            }
            return Ok(Some(expr));
        }
        Ok(None)
    }
    // factor         → unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<Option<Expr>> {
        if let Some(mut expr) = self.unary()? {
            while self.match_type(vec![TokenType::Slash, TokenType::Star]) {
                let operator = self.previous();
                if let Some(right) = self.unary()? {
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    }
                }
            }

            return Ok(Some(expr));
        }
        Ok(None)
    }
    // unary          → ( "!" | "-" ) unary
    //                | primary ;
    fn unary(&mut self) -> Result<Option<Expr>> {
        if let Some(mut expr) = self.primary()? {
            if self.match_type(vec![TokenType::Bang, TokenType::Minus]) {
                let operator = self.previous();
                if let Some(right) = self.unary()? {
                    expr = Expr::Unary {
                        operator,
                        right: Box::new(right),
                    }
                }
            }
            return Ok(Some(expr));
        }
        Ok(None)
    }
    // primary        → NUMBER | STRING | "true" | "false" | "nil"
    //                | "(" expression ")";
    fn primary(&mut self) -> Result<Option<Expr>> {
        if self.match_type(vec![TokenType::FALSE]) {
            return Ok(Some(Expr::Literal {
                value: Some("false".into()),
            }));
        }
        if self.match_type(vec![TokenType::TRUE]) {
            return Ok(Some(Expr::Literal {
                value: Some("true".into()),
            }));
        }
        if self.match_type(vec![TokenType::NIL]) {
            return Ok(Some(Expr::Literal {
                value: Some("nil".into()),
            }));
        }
        if self.match_type(vec![TokenType::NumberLit, TokenType::StringLit]) {
            return Ok(Some(Expr::Literal {
                value: self.previous().literal,
            }));
        }

        if self.match_type(vec![TokenType::LeftParen]) {
            if let Some(expr) = self.expression()? {
                self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
                return Ok(Some(Expr::Grouping {
                    expression: Box::new(expr),
                }));
            }
        }

        let token = self.peek();
        if matches!(token.ty, TokenType::EOF) {
            Ok(None)
        } else {
            Err(Report::new(BaseError::TokenError {
                line: token.line,
                location: token.lexeme,
                message: "Expected expression.".into(),
            }))
        }
    }
}
