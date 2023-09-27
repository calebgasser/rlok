use super::error_handler::ParserError;
use super::expression::Expr;
use super::lit::LitType;
use super::statement::Statement;
use super::tokens::{Token, TokenType};
use color_eyre::eyre::{Report, Result};

pub struct Parser {
    tokens: Vec<Token>,
    current: i32,
    debug: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, debug: bool) -> Result<Self> {
        Ok(Parser {
            tokens,
            current: 0,
            debug,
        })
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
            let token = self.previous();
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
            match self.declaration() {
                Ok(declaration) => {
                    if let Some(dec) = declaration {
                        if self.debug {
                            println!("{:#?}", dec);
                        }
                        statements.push(dec.clone())
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(Some(statements))
    }

    fn declaration(&mut self) -> Result<Option<Statement>> {
        if self.match_type(vec![TokenType::FUN]) {
            return Ok(Some(self.function_declaration("function".into())?));
        } else if self.match_type(vec![TokenType::VAR]) {
            return self.var_declaration();
        } else {
            self.statement()
        }
    }

    fn function_declaration(&mut self, kind: String) -> Result<Statement> {
        let name = self.consume(TokenType::Ident, &format!("Expect {} name.", kind))?;
        let _ = self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.)", kind),
        )?;
        let mut parameters = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(Report::new(ParserError::MaxArguments(self.peek())));
                }
                parameters.push(self.consume(TokenType::Ident, "Expect parameter name.")?);
                if !self.match_type(vec![TokenType::Comma]) {
                    break;
                }
            }
        }
        let _ = self.consume(TokenType::RightParen, "Expect ')' after parameters");
        let _ = self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body", kind),
        );
        let body = self.block_statement()?;
        Ok(Statement::Function {
            name,
            params: parameters,
            body,
        })
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
        let _ = self.consume(
            TokenType::Semicolon,
            "Expected ';' after unintialized variable.",
        )?;
        return Ok(Some(Statement::Var {
            name,
            expression: Some(Expr::Literal {
                value: Some(LitType::Nil),
            }),
        }));
    }

    fn statement(&mut self) -> Result<Option<Statement>> {
        if self.match_type(vec![TokenType::LeftBrace]) {
            return Ok(Some(Statement::Block {
                statements: self.block_statement()?,
            }));
        } else if self.match_type(vec![TokenType::IF]) {
            Ok(Some(self.if_statement()?))
        } else if self.match_type(vec![TokenType::PRINT]) {
            self.print_statement()
        } else if self.match_type(vec![TokenType::RETURN]) {
            self.return_statement()
        } else if self.match_type(vec![TokenType::WHILE]) {
            self.while_statement()
        } else if self.match_type(vec![TokenType::FOR]) {
            self.for_statement()
        } else {
            self.expression_statement()
        }
    }

    fn return_statement(&mut self) -> Result<Option<Statement>> {
        let keyword = self.previous();
        if !self.match_type(vec![TokenType::Semicolon]) {
            if let Some(value) = self.expression()? {
                let _ = self.consume(TokenType::Semicolon, "Expect ';' after return value");
                return Ok(Some(Statement::Return { keyword, value }));
            }
        }
        Ok(None)
    }

    fn for_statement(&mut self) -> Result<Option<Statement>> {
        let _ = self.consume(TokenType::LeftParen, "Expected '(' after 'for'.");
        let initializer: Option<Statement>;
        let mut condition: Option<Expr> = None;
        let mut increment: Option<Expr> = None;
        if self.match_type(vec![TokenType::Semicolon]) {
            initializer = None;
        } else if self.match_type(vec![TokenType::VAR]) {
            initializer = self.var_declaration()?;
        } else {
            initializer = self.expression_statement()?;
        }

        if !self.check(TokenType::Semicolon) {
            condition = self.expression()?;
        }

        let _ = self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

        if !self.check(TokenType::RightParen) {
            increment = self.expression()?;
        }

        let _ = self.consume(TokenType::RightParen, "Expect ')' after for clause.");

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            if let Some(bdy) = body {
                body = Some(Statement::Block {
                    statements: vec![
                        Box::new(bdy),
                        Box::new(Statement::Expression { expression: inc }),
                    ],
                });
            }
        }

        if let Some(con) = condition {
            if let Some(bdy) = body {
                body = Some(Statement::While {
                    condition: con,
                    body: Box::new(bdy),
                });
            }
        } else {
            if let Some(bdy) = body {
                body = Some(Statement::While {
                    condition: Expr::Literal {
                        value: Some(LitType::Bool(true)),
                    },
                    body: Box::new(bdy),
                });
            }
        }

        if let Some(init) = initializer {
            if let Some(bdy) = body {
                body = Some(Statement::Block {
                    statements: vec![Box::new(init), Box::new(bdy)],
                });
            }
        }

        return Ok(body);
    }

    fn while_statement(&mut self) -> Result<Option<Statement>> {
        let _ = self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        if let Some(condition) = self.expression()? {
            let _ = self.consume(TokenType::RightParen, "Expected ')' after condition.");
            if let Some(body) = self.statement()? {
                return Ok(Some(Statement::While {
                    condition,
                    body: Box::new(body),
                }));
            }
            return Err(Report::new(ParserError::WhileMissingBody(condition)));
        }
        return Err(Report::new(ParserError::WhileMissingCondition(
            self.previous(),
        )));
    }

    fn if_statement(&mut self) -> Result<Statement> {
        let _ = self.consume(TokenType::LeftParen, "Expected '(' after 'if'.");
        if let Some(condition) = self.expression()? {
            let _ = self.consume(TokenType::RightParen, "Expected ')' after if condition.");
            if let Some(then_branch) = self.statement()? {
                if self.match_type(vec![TokenType::ELSE]) {
                    if let Some(els) = self.statement()? {
                        return Ok(Statement::If {
                            condition,
                            then_branch: Box::new(then_branch),
                            else_branch: Some(Box::new(els)),
                        });
                    }
                } else {
                    return Ok(Statement::If {
                        condition,
                        then_branch: Box::new(then_branch),
                        else_branch: None,
                    });
                }
            }
            return Err(Report::new(ParserError::MissingThenBranch(condition)));
        }
        return Err(Report::new(ParserError::MissingIfCondition(
            self.previous(),
        )));
    }

    fn block_statement(&mut self) -> Result<Vec<Box<Statement>>> {
        let mut statements: Vec<Box<Statement>> = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_end() {
            if let Some(dec) = self.declaration()? {
                statements.push(Box::new(dec));
            }
        }
        let _ = self.consume(TokenType::RightBrace, "Expected '}' after block.")?;
        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<Option<Statement>> {
        if let Some(expr) = self.expression()? {
            let _ = self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
            return Ok(Some(Statement::Print { expression: expr }));
        }
        Err(Report::new(ParserError::PrintNoExpression(self.previous())))
    }

    fn expression_statement(&mut self) -> Result<Option<Statement>> {
        if let Some(expr) = self.expression()? {
            return Ok(Some(Statement::Expression { expression: expr }));
        }
        Ok(None)
    }

    // expression     → comma ;
    fn expression(&mut self) -> Result<Option<Expr>> {
        if let Some(expr) = self.assignment()? {
            return Ok(Some(expr));
        }
        Ok(None)
    }

    fn assignment(&mut self) -> Result<Option<Expr>> {
        if let Some(expr) = self.logic_or()? {
            if self.match_type(vec![TokenType::Equal]) {
                let equals = self.previous();
                if let Some(value) = self.assignment()? {
                    if let Expr::Variable { name } = expr {
                        if matches!(self.peek().ty, TokenType::Semicolon) {
                            let _ = self.consume(
                                TokenType::Semicolon,
                                "Expect ';' after variable assignment.",
                            )?;
                        }
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

    fn logic_or(&mut self) -> Result<Option<Expr>> {
        if let Some(expr) = self.logic_and()? {
            while self.match_type(vec![TokenType::OR]) {
                let operator = self.previous();
                if let Some(right) = self.logic_and()? {
                    return Ok(Some(Expr::Logcial {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    }));
                }
                return Err(Report::new(ParserError::LogicOrMissingRight(expr)));
            }
            return Ok(Some(expr));
        }
        Ok(None)
    }

    fn logic_and(&mut self) -> Result<Option<Expr>> {
        if let Some(expr) = self.equality()? {
            while self.match_type(vec![TokenType::AND]) {
                let operator = self.previous();
                if let Some(right) = self.equality()? {
                    return Ok(Some(Expr::Logcial {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    }));
                }
                return Err(Report::new(ParserError::LogicAndMissingRight(expr)));
            }
            return Ok(Some(expr));
        }
        Ok(None)
    }

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

    fn unary(&mut self) -> Result<Option<Expr>> {
        if let Some(mut expr) = self.call()? {
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

    fn call(&mut self) -> Result<Option<Expr>> {
        if let Some(mut expr) = self.primary()? {
            loop {
                if self.match_type(vec![TokenType::LeftParen]) {
                    expr = self.finish_call(expr)?;
                } else {
                    break;
                }
            }
            return Ok(Some(expr));
        }
        Ok(None)
    }

    fn finish_call(&mut self, expr: Expr) -> Result<Expr> {
        let mut arguments = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if let Some(ex) = self.expression()? {
                    if arguments.len() >= 255 {
                        return Err(Report::new(ParserError::MaxArguments(self.peek())));
                    }
                    arguments.push(Box::new(ex));
                }
                if !self.match_type(vec![TokenType::Comma]) {
                    break;
                }
            }
        }
        let paren = self.consume(TokenType::RightParen, "Expected ')' after arguments.")?;
        let _ = self.consume(TokenType::Semicolon, "Expected ';' after function call.")?;
        return Ok(Expr::Call {
            callee: Box::new(expr),
            paren,
            arguments,
        });
    }

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
