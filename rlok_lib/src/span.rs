use super::tokens::{Token, TokenType};
use tracing::trace;

#[derive(Debug, Clone)]
pub struct Span {
    first: i32,
    last: i32,
}

pub struct SpanParser;

impl SpanParser {
    fn get_underline(space_before: usize, num_underline: usize) -> String {
        let mut output = String::from("");
        for _ in 0..space_before {
            output = format!("{} ", output);
        }
        for _ in 0..num_underline {
            output = format!("{}^", output);
        }
        format!("{}--- Here", output)
    }
    pub fn parse(span: Span, tokens: Vec<Token>) -> String {
        let mut underline_space_before = 0;
        let mut num_underline = 0;
        let mut start = span.first();
        let mut end = span.last();
        loop {
            if let Some(token) = tokens.get((start - 1) as usize) {
                if matches!(token.ty, TokenType::NewLine) {
                    break;
                } else {
                    start -= 1;
                }
            } else {
                break;
            }
        }
        loop {
            if let Some(token) = tokens.get((end + 1) as usize) {
                if matches!(token.ty, TokenType::NewLine) || matches!(token.ty, TokenType::EOF) {
                    break;
                } else {
                    end += 1;
                }
            } else {
                break;
            }
        }

        if let Some(span_tokens) = tokens.get((span.first() - 1) as usize..span.last() as usize) {
            span_tokens
                .into_iter()
                .reduce(|_, token| {
                    num_underline += token.lexeme.len();
                    token
                })
                .unwrap();
        }

        if let Some(span_tokens) = tokens.get(start as usize..span.first() as usize) {
            let _output = span_tokens.into_iter().fold(String::new(), |accu, token| {
                if accu.len() <= 0 {
                    if let Some(prev_token) = tokens.get(start as usize) {
                        let prepend = format!("\n{}| {}", token.line, prev_token);
                        underline_space_before += prepend.len();
                        format!("\n{}|{}{}", token.line - 1, prepend, token.lexeme)
                    } else {
                        let prepend = format!("\n{}| ", token.line);
                        underline_space_before += prepend.len();
                        format!("\n{}|{}{}", token.line - 1, prepend, token.lexeme)
                    }
                } else {
                    underline_space_before += token.lexeme.len();
                    format!("{}{}", accu, token.lexeme)
                }
            });
        }
        if let Some(span_tokens) = tokens.get(start as usize..end as usize) {
            let output = span_tokens.into_iter().fold(String::new(), |accu, token| {
                if accu.len() <= 0 {
                    if let Some(prev_token) = tokens.get(start as usize) {
                        let prepend = format!("\n{}| {}", token.line, prev_token);
                        format!("\n{}|{}{}", token.line - 1, prepend, token.lexeme)
                    } else {
                        let prepend = format!("\n{}| ", token.line);
                        format!("\n{}|{}{}", token.line - 1, prepend, token.lexeme)
                    }
                } else {
                    format!("{}{}", accu, token.lexeme)
                }
            });
            if let Some(last_token) = tokens.get(end as usize) {
                let prepend = format!("{}|", last_token.line + 1);
                format!(
                    "{}{}\n{}{}",
                    output,
                    last_token,
                    prepend,
                    SpanParser::get_underline(
                        underline_space_before - prepend.len(),
                        num_underline
                    )
                )
            } else {
                format!(
                    "{}{}",
                    output,
                    SpanParser::get_underline(underline_space_before, num_underline)
                )
            }
        } else {
            format!("TOKEN OUT OF BOUNDS")
        }
    }
}

impl Span {
    pub fn new(first: i32) -> Self {
        Span {
            first,
            last: Default::default(),
        }
    }

    pub fn set_first_and_last(&mut self, first: i32, last: i32) -> &mut Span {
        self.first = first;
        self.last = last;
        self
    }

    pub fn set_first(mut self, first: i32) -> Span {
        self.first = first;
        self
    }

    pub fn set_last(&mut self, last: i32) -> &Span {
        self.last = last;
        self
    }

    pub fn done(&self) -> Span {
        Span {
            first: self.first,
            last: self.last,
        }
    }

    pub fn first(&self) -> i32 {
        self.first
    }

    pub fn last(&self) -> i32 {
        self.last
    }
}
