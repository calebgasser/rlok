use super::environment::Environment;
use super::error_handler::RuntimeError;
use super::expression::Expr;
use super::lit::LitType;
use super::lox_callable::{Callable, Clock, LoxCallable, LoxFunction};
use super::parser::Parser;
use super::scanner::Scanner;
use super::span::Span;
use super::statement::Statement;
use super::tokens::{Token, TokenType};
use color_eyre::eyre::{Report, Result};
use std::fs;
use std::io;
use std::io::Write;
use tracing::{span, trace, Level};

#[derive(Debug)]
pub struct Interpreter {
    pub globals: Environment,
    pub environment: Environment,
    parser: Option<Parser>,
    is_repl: bool,
}

impl Interpreter {
    #[tracing::instrument]
    pub fn build() -> Self {
        env_logger::init();
        let mut globals = Environment::new(None);
        globals.define(
            "clock".into(),
            LitType::Callable(LoxCallable::Clock(Clock::new("clock".into(), None))),
        );
        Interpreter {
            parser: None,
            globals: globals.clone(),
            environment: globals.clone(),
            is_repl: false,
        }
    }

    pub fn start(&mut self, args: Vec<String>) -> Result<()> {
        trace!("Starting Interpreter");
        if args.len() == 2 {
            self.run_file(&args[1])?;
        } else {
            self.is_repl = true;
            self.run_prompt()?;
        }
        Ok(())
    }

    fn run(&mut self, contents: String) -> Result<()> {
        let mut scanner = Scanner::build(contents);
        let tokens = scanner.scan_tokens()?;
        self.parser = Some(Parser::new(tokens.clone())?);
        if let Some(ref mut parser) = self.parser {
            match parser.parse() {
                Ok(ast) => {
                    let span = span!(Level::TRACE, "interpreter");
                    let _enter = span.enter();
                    if let Some(ast) = ast {
                        for stmt in &ast {
                            trace!(statement = %stmt, "Processing statement.");
                            match self.evaluate_statement(stmt.clone()) {
                                Ok(output) => {
                                    if self.is_repl {
                                        if let Some(out) = output {
                                            Self::print_lit(out);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                }
                            }
                        }
                    } else {
                        eprintln!("Failed to generate AST")
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }
        Ok(())
    }

    fn run_file(&mut self, file: &str) -> Result<()> {
        let contents = fs::read_to_string(file)?;
        self.run(contents)?;
        Ok(())
    }

    fn run_prompt(&mut self) -> Result<()> {
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
                match self.run(buffer) {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }

    pub fn get_span(&self, span: Span) -> String {
        if let Some(parser) = &self.parser {
            parser.get_span(span)
        } else {
            String::from("NO SPAN")
        }
    }

    fn literal_expr(&self, expr: Expr) -> Result<LitType> {
        if let Expr::Literal { span: _, value } = expr.clone() {
            if let Some(val) = value {
                let span = span!(Level::TRACE, "literal expression");
                let _enter = span.enter();
                trace!(value = %val);
                return Ok(val);
            }
        }
        Err(Report::new(RuntimeError::InvalidLiteral(expr)))
    }

    fn grouping_expr(&mut self, expr: Expr) -> Result<LitType> {
        if let Expr::Grouping {
            span: _,
            expression,
        } = expr
        {
            let span = span!(Level::TRACE, "grouping expression");
            let _enter = span.enter();
            trace!(expr = %expression);
            return self.evaluate_expr(*expression);
        }
        Err(Report::new(RuntimeError::InvalidGrouping(expr)))
    }

    fn unary_expr(&mut self, expr: Expr) -> Result<LitType> {
        if let Expr::Unary {
            span: _,
            operator,
            right,
        } = expr.clone()
        {
            let span = span!(Level::TRACE, "unary expression");
            let _enter = span.enter();
            let right = self.evaluate_expr(*right)?;
            match operator.ty {
                TokenType::Minus => {
                    if let LitType::Float(f) = right {
                        trace!(value = f);
                        return Ok(LitType::Float(-f));
                    }
                }
                TokenType::Bang => match right {
                    LitType::Bool(b) => {
                        trace!(value = b);
                        return Ok(LitType::Bool(!b));
                    }
                    LitType::Nil => {
                        trace!(value = "Nil");
                        return Ok(LitType::Bool(true));
                    }
                    _ => return Err(Report::new(RuntimeError::RighthandBoolorNil(expr))),
                },
                _ => return Err(Report::new(RuntimeError::UnaryExpects(expr))),
            }
        }
        Err(Report::new(RuntimeError::InvalidUnary(expr)))
    }

    fn binary_expr(&mut self, expr: Expr) -> Result<LitType> {
        if let Expr::Binary {
            span: _,
            left,
            operator,
            right,
        } = expr.clone()
        {
            let left = self.evaluate_expr(*left)?;
            let right = self.evaluate_expr(*right)?;
            let span = span!(Level::TRACE, "binary expression");
            let _enter = span.enter();
            trace!(left = %left, operator = %operator.ty, right=%right);
            if let LitType::Float(r) = right {
                if let LitType::Float(l) = left {
                    match operator.ty {
                        TokenType::Plus => return Ok(LitType::Float(l + r)),
                        TokenType::Minus => return Ok(LitType::Float(l - r)),
                        TokenType::Slash => {
                            if r == 0.0 {
                                return Err(Report::new(RuntimeError::DivideByZero(expr)));
                            } else {
                                return Ok(LitType::Float(l / r));
                            }
                        }
                        TokenType::Star => return Ok(LitType::Float(l * r)),
                        TokenType::Less => return Ok(LitType::Bool(l < r)),
                        TokenType::LessEqual => return Ok(LitType::Bool(l <= r)),
                        TokenType::Greater => return Ok(LitType::Bool(l > r)),
                        TokenType::GreaterEqual => return Ok(LitType::Bool(l >= r)),
                        TokenType::EqualEqual => return Ok(LitType::Bool(l == r)),
                        _ => {
                            return Err(Report::new(RuntimeError::InvalidNumerical(expr, operator)))
                        }
                    }
                }
            }
            if let LitType::Str(r) = right {
                if let LitType::Str(l) = left {
                    match operator.ty {
                        TokenType::Plus => return Ok(LitType::Str(format!("{}{}", l, r))),
                        _ => return Err(Report::new(RuntimeError::InvalidStringConcat(expr))),
                    }
                }
            }
            return Err(Report::new(RuntimeError::BinaryTypeMismatch(expr)));
        }
        Err(Report::new(RuntimeError::InvalidBinaryExpr(expr)))
    }

    fn var_expr(&self, expr: Expr) -> Result<LitType> {
        match expr {
            Expr::Variable { span, name } => {
                if let Some(val) = self
                    .environment
                    .get(name.clone(), self.get_span(span.clone()))?
                {
                    let span = span!(Level::TRACE, "var expression");
                    let _enter = span.enter();
                    trace!(name = %name, val = %val);
                    return Ok(val);
                }
                return Err(Report::new(RuntimeError::UndefinedVariable(
                    name.lexeme,
                    self.get_span(span),
                )));
            }
            _ => Err(Report::new(RuntimeError::ExpressionNotVariable(expr))),
        }
    }

    fn logical_expr(&mut self, left: Expr, operator: TokenType, right: Expr) -> Result<LitType> {
        let left = self.evaluate_expr(left)?;
        let span = span!(Level::TRACE, "logical expression");
        let _enter = span.enter();
        trace!(left = %left);
        if matches!(operator, TokenType::OR) {
            if Self::is_truthy(left.clone()) {
                return Ok(left);
            };
        } else {
            if !Self::is_truthy(left.clone()) {
                return Ok(left);
            };
        }
        return self.evaluate_expr(right);
    }

    fn print_lit(lit: LitType) {
        match lit {
            LitType::Float(flt) => println!("{}", flt),
            LitType::Str(str) => println!("{}", str),
            LitType::Bool(bl) => println!("{}", bl),
            LitType::Callable(call) => match call {
                LoxCallable::Function(func) => println!("{}", func.to_string()),
                LoxCallable::Clock(clock) => println!("{}", clock.to_string()),
            },
            LitType::Nil => println!("nil"),
        }
    }

    fn is_truthy(lit: LitType) -> bool {
        match lit {
            LitType::Float(flt) => {
                if flt > 0.0 {
                    true
                } else {
                    false
                }
            }
            LitType::Str(str) => {
                if str.len() > 0 {
                    true
                } else {
                    false
                }
            }
            LitType::Bool(bl) => bl,
            LitType::Nil => false,
            _ => false,
        }
    }

    fn while_statement(&mut self, condition: Expr, body: Statement) -> Result<Option<LitType>> {
        let span = span!(Level::TRACE, "while statement");
        let _enter = span.enter();
        while Self::is_truthy(self.evaluate_expr(condition.clone())?) {
            trace!(body = %body, "While...");
            let output = self.evaluate_statement(body.clone())?;
            if let Some(output) = output {
                return Ok(Some(output));
            }
        }
        Ok(None)
    }

    fn if_statement(
        &mut self,
        condition: Expr,
        then_condition: Statement,
        else_condition: Option<Statement>,
    ) -> Result<Option<LitType>> {
        let span = span!(Level::TRACE, "if statement");
        let _enter = span.enter();
        trace!(condition = %condition);
        if Self::is_truthy(self.evaluate_expr(condition)?) {
            return self.evaluate_statement(then_condition);
        } else if let Some(els) = else_condition {
            return self.evaluate_statement(els);
        }
        Ok(None)
    }

    fn var_statement(&mut self, stmt: Statement) -> Result<()> {
        if let Statement::Var {
            span: _,
            name,
            expression,
        } = stmt.clone()
        {
            if let Some(expr) = expression {
                let span_trace = span!(Level::TRACE, "var statement");
                let _enter = span_trace.enter();
                let value = self.evaluate_expr(expr.clone())?;
                trace!(name = %name, expr = %expr, value = %value.clone());
                self.environment.define(name.lexeme, value);
                return Ok(());
            }
            return Err(Report::new(RuntimeError::StatementMissingExpression(stmt)));
        }
        Err(Report::new(RuntimeError::UnexpectedStatement(stmt)))
    }

    pub fn block_statement(
        &mut self,
        statements: Vec<Box<Statement>>,
        environment: Environment,
    ) -> Result<Option<LitType>> {
        let span_trace = span!(Level::TRACE, "b>");
        let _enter = span_trace.enter();
        self.environment = environment;
        trace!(env = %self.environment, "Starting block statement");
        for stmt in statements {
            trace!(statement = %stmt, "Processing statement in block");
            self.evaluate_statement(*stmt.clone())?;
        }
        Ok(None)
    }

    fn function_statement(
        &mut self,
        name: Token,
        params: Vec<Token>,
        body: Vec<Box<Statement>>,
        span: Span,
    ) -> Result<()> {
        let span_trace = span!(Level::TRACE, "function statement");
        let _enter = span_trace.enter();
        trace!(name = %name);
        let stmt = Statement::Function {
            span,
            name: name.clone(),
            params,
            body,
        };
        let function = LitType::Callable(LoxCallable::Function(LoxFunction::new(
            name.lexeme.clone(),
            Some(stmt.clone()),
        )));
        self.environment.define(name.lexeme, function);
        Ok(())
    }

    fn return_statement(&mut self, _keyword: Token, value: Expr) -> Result<LitType> {
        let span = span!(Level::TRACE, "return statement");
        let _enter = span.enter();
        trace!(value = %value);
        Err(Report::new(RuntimeError::Return(
            self.evaluate_expr(value)?,
        )))
    }

    fn evaluate_statement(&mut self, stmt: Statement) -> Result<Option<LitType>> {
        match stmt.clone() {
            Statement::Print {
                span: _,
                expression,
            } => {
                let value = self.evaluate_expr(expression)?;
                trace!(value = %value, "Print lit statement");
                Self::print_lit(value);
                Ok(None)
            }
            Statement::Expression {
                span: _,
                expression,
            } => {
                return Ok(Some(self.evaluate_expr(expression)?));
            }
            Statement::Var {
                span: _,
                name: _,
                expression: _,
            } => {
                self.var_statement(stmt)?;
                return Ok(None);
            }
            Statement::Block {
                span: _,
                statements,
            } => {
                return Ok(self.block_statement(
                    statements,
                    Environment::new(Some(Box::new(self.environment.clone()))),
                )?);
            }
            Statement::If {
                span: _,
                condition,
                then_branch,
                else_branch,
            } => {
                if let Some(els) = else_branch {
                    return self.if_statement(condition, *then_branch, Some(*els));
                } else {
                    return self.if_statement(condition, *then_branch, None);
                }
            }
            Statement::While {
                span: _,
                condition,
                body,
            } => {
                return self.while_statement(condition, *body);
            }
            Statement::Function {
                span,
                name,
                params,
                body,
            } => {
                self.function_statement(name, params, body, span)?;
                Ok(None)
            }
            Statement::Return {
                span: _,
                keyword,
                value,
            } => {
                return Ok(Some(self.return_statement(keyword, value)?));
            }
        }
    }

    fn assign_expr(&mut self, name: Token, value: Expr, span: Span) -> Result<LitType> {
        if let Expr::Literal { span: _, ref value } = value {
            if let Some(val) = value {
                let span_tracing = span!(Level::TRACE, "assign expression");
                let _enter = span_tracing.enter();
                trace!(name = %name.clone(), value = %val.clone(), "assigning");
                self.environment
                    .assign(name.clone(), val.clone(), self.get_span(span))?;
                return Ok(val.clone());
            }
        } else {
            let span_tracing = span!(Level::TRACE, "assign expression eval");
            let _enter = span_tracing.enter();
            trace!(value = %value.clone());
            let val = self.evaluate_expr(value.clone())?;
            trace!(name = %name.clone(), value = %val.clone(), "assigning");
            self.environment
                .assign(name.clone(), val.clone(), self.get_span(span))?;
            return Ok(val.clone());
        }
        Err(Report::new(RuntimeError::InvalidAssignmentTarget(
            name, value,
        )))
    }

    fn call_expr(
        &mut self,
        callee: Expr,
        paren: Token,
        arguments: Vec<Box<Expr>>,
    ) -> Result<LitType> {
        let callee = self.evaluate_expr(callee)?;
        let mut args = Vec::new();
        for arg in arguments.clone() {
            args.push(self.evaluate_expr(*arg)?);
        }
        if let LitType::Callable(call) = callee.clone() {
            match call {
                LoxCallable::Function(func) => {
                    if arguments.len() != func.arity() {
                        return Err(Report::new(RuntimeError::IncorrectArgumentCount(
                            func.arity(),
                            arguments.len(),
                        )));
                    }
                    let span = span!(Level::TRACE, "call expression");
                    let _enter = span.enter();
                    trace!(callee = func.as_string(), "Calling function");
                    return Ok(func.call(self, arguments)?);
                }
                LoxCallable::Clock(clock) => {
                    if arguments.len() != clock.arity() {
                        return Err(Report::new(RuntimeError::IncorrectArgumentCount(
                            clock.arity(),
                            arguments.len(),
                        )));
                    }
                    let span = span!(Level::TRACE, "call expression");
                    let _enter = span.enter();
                    trace!(callee = clock.as_string(), "Calling function");
                    return Ok(clock.call(self, arguments)?);
                }
            }
        }
        Err(Report::new(RuntimeError::NotCallable(paren)))
    }

    pub fn evaluate_expr(&mut self, expr: Expr) -> Result<LitType> {
        match &expr {
            Expr::Binary {
                span: _,
                left: _,
                operator: _,
                right: _,
            } => Ok(self.binary_expr(expr)?),
            Expr::Grouping {
                span: _,
                expression: _,
            } => Ok(self.grouping_expr(expr)?),
            Expr::Unary {
                span: _,
                operator: _,
                right: _,
            } => Ok(self.unary_expr(expr)?),
            Expr::Literal { span: _, value: _ } => Ok(self.literal_expr(expr)?),
            Expr::Variable { span: _, name: _ } => Ok(self.var_expr(expr)?),
            Expr::Assign { span, name, value } => {
                Ok(self.assign_expr(name.clone(), *value.clone(), span.clone())?)
            }
            Expr::Logcial {
                span: _,
                left,
                operator,
                right,
            } => Ok(self.logical_expr(*left.clone(), operator.ty.clone(), *right.clone())?),
            Expr::Call {
                span: _,
                callee,
                paren,
                arguments,
            } => Ok(self.call_expr(*callee.clone(), paren.clone(), arguments.clone())?),
        }
    }
}
