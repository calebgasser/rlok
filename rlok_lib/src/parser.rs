use super::error_handler::ParserError;
use super::expression::Expr;
use super::lit::LitType;
use super::statement::Statement;
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
// comma          → comparison ( "," comparison )*;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" ;

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

    fn consume(&mut self, ty: TokenType, error_message: &str) -> Result<Token> {
        if self.check(ty) {
            Ok(self.advance())
        } else {
            let token = self.peek();
            Err(Report::new(ParserError::ConsumeTokenError {
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
            return self.peek().ty == ty;
        }
    }

    fn match_type(&mut self, types: Vec<TokenType>) -> bool {
        for ty in types {
            if self.check(ty.clone()) {
                self.advance();
                return true;
            }
        }
        false
    }

    pub fn parse(&mut self) -> Result<Option<Vec<Statement>>> {
        let mut statements: Vec<Statement> = Vec::new();
        while !self.is_end() {
            if let Some(dec) = self.declaration()? {
                statements.push(dec.clone());
            }
        }
        Ok(Some(statements))
    }

    fn declaration(&mut self) -> Result<Option<Statement>> {
        if self.match_type(vec![TokenType::VAR]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn var_declaration(&mut self) -> Result<Option<Statement>> {
        let name = self.consume(TokenType::Ident, "Expected variable name.")?;
        if self.match_type(vec![TokenType::Equal]) {
            if let Some(expr) = self.expression()? {
                let _ = self.consume(
                    TokenType::Semicolon,
                    "Expect ';' after variable declaration.",
                )?;
                return Ok(Some(Statement::Var {
                    name,
                    expression: Some(expr),
                }));
            }
            return Err(Report::new(ParserError::VarMissingExpr(name)));
        }
        Err(Report::new(ParserError::VarDeclartionError))
    }

    fn statement(&mut self) -> Result<Option<Statement>> {
        if self.match_type(vec![TokenType::PRINT]) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Option<Statement>> {
        if let Some(expr) = self.expression()? {
            let _ = self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
            return Ok(Some(Statement::Print { expression: expr }));
        }
        Err(Report::new(ParserError::PrintNoExpression))
    }

    fn expression_statement(&mut self) -> Result<Option<Statement>> {
        if let Some(expr) = self.expression()? {
            let _ = self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
            return Ok(Some(Statement::Expression { expression: expr }));
        }
        Err(Report::new(ParserError::ExpressionNoExpression))
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
        if let Some(mut expr) = self.assignment()? {
            while self.match_type(vec![TokenType::Comma]) {
                let operator = self.previous();
                if let Some(right) = self.assignment()? {
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

    fn assignment(&mut self) -> Result<Option<Expr>> {
        if let Some(expr) = self.equality()? {
            if self.match_type(vec![TokenType::Equal]) {
                let equals = self.previous();
                if let Some(value) = self.assignment()? {
                    if let Expr::Variable { name } = expr {
                        return Ok(Some(Expr::Assign {
                            name,
                            value: Box::new(value),
                        }));
                    }
                    return Err(Report::new(ParserError::UnexpectedAssignmentTarget(equals)));
                }
                return Err(Report::new(ParserError::InvalidAssignmentTarget(equals)));
            }
            return Ok(Some(expr));
        }
        Ok(None)
    }

    // equality       → comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Option<Expr>> {
        if let Some(expr) = self.comparison()? {
            while self.match_type(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
                let operator = self.previous();
                if let Some(right) = self.comparison()? {
                    return Ok(Some(Expr::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    }));
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
                value: Some(LitType::Bool(false)),
            }));
        }
        if self.match_type(vec![TokenType::TRUE]) {
            return Ok(Some(Expr::Literal {
                value: Some(LitType::Bool(true)),
            }));
        }
        if self.match_type(vec![TokenType::NIL]) {
            return Ok(Some(Expr::Literal {
                value: Some(LitType::Nil),
            }));
        }
        if self.match_type(vec![TokenType::NumberLit]) {
            return Ok(Some(Expr::Literal {
                value: Some(LitType::Float(
                    self.previous().literal.unwrap().parse::<f32>().unwrap(),
                )),
            }));
        }
        if self.match_type(vec![TokenType::StringLit]) {
            return Ok(Some(Expr::Literal {
                value: Some(LitType::Str(self.previous().literal.unwrap())),
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

        if self.match_type(vec![TokenType::Ident]) {
            return Ok(Some(Expr::Variable {
                name: self.previous(),
            }));
        }

        let token = self.peek();
        if matches!(token.ty, TokenType::EOF) {
            Ok(None)
        } else {
            Err(Report::new(ParserError::PrimaryTokenError {
                line: token.line,
                location: token.lexeme,
                message: "Expected expression.".into(),
            }))
        }
    }
}
