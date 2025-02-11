use crate::error::SyntaxError;
use crate::Result;
use std::fmt::Debug;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum FilterExpr<T> {
    And(Box<FilterExpr<T>>, Box<FilterExpr<T>>),
    Or(Box<FilterExpr<T>>, Box<FilterExpr<T>>),
    Not(Box<FilterExpr<T>>),
    Value(T),
    Raw(String), // Unparsed value string
}
use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")] // Skip whitespace
pub enum Token {
    #[token(" & ")]
    And,

    #[token(" | ")]
    Or,

    #[token(" ; ")]
    SemiColon,

    #[token("!")]
    Not,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[regex("[^&|;!()\\s]+")]
    Value,
}

pub struct ExprParser<'a> {
    lexer: logos::Lexer<'a, Token>,
    current_token: Option<Result<Token, SyntaxError>>,
    current_slice: &'a str,
    input: &'a str,
}

impl<'a> ExprParser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Token::lexer(input);
        let current_token = match lexer.next() {
            Some(Ok(token)) => Some(Ok(token)),
            Some(Err(_)) => Some(Err(SyntaxError {
                src: input.to_string(),
                span: (lexer.span().start, lexer.span().len()).into(),
                message: format!("Invalid token: {}", lexer.slice()),
            })),
            None => None,
        };
        let current_slice = lexer.slice();

        Self {
            lexer,
            current_token,
            current_slice,
            input,
        }
    }

    fn advance(&mut self) {
        self.current_token = match self.lexer.next() {
            Some(Ok(token)) => Some(Ok(token)),
            Some(Err(_)) => Some(Err(SyntaxError {
                src: self.input.to_string(),
                span: (self.lexer.span().start, self.lexer.span().len()).into(),
                message: format!("Invalid token: {}", self.lexer.slice()),
            })),
            None => None,
        };
        self.current_slice = self.lexer.slice();
    }

    fn expect(&mut self, expected: Token) -> Result<(), SyntaxError> {
        match &self.current_token {
            Some(Ok(token)) if token == &expected => {
                self.advance();
                Ok(())
            }
            Some(Ok(token)) => Err(SyntaxError {
                src: self.input.to_string(),
                span: (self.lexer.span().start, self.lexer.span().len()).into(),
                message: format!("Expected {:?}, found {:?}", expected, token),
            }),
            Some(Err(e)) => Err(e.clone()),
            None => Err(SyntaxError {
                src: self.input.to_string(),
                span: (self.input.len(), 0).into(),
                message: format!("Expected {:?}, found end of input", expected),
            }),
        }
    }

    pub fn parse<T>(&mut self) -> Result<FilterExpr<T>, SyntaxError>
    where
        T: FromStr,
    {
        let expr = self.parse_or()?;

        if self.current_token.is_some() {
            Err(SyntaxError {
                src: self.input.to_string(),
                span: (self.lexer.span().start, self.lexer.span().len()).into(),
                message: format!("Unexpected token: {:?}", self.current_token),
            })
        } else {
            Ok(expr)
        }
    }

    fn parse_or<T>(&mut self) -> Result<FilterExpr<T>, SyntaxError>
    where
        T: FromStr,
    {
        let mut left = self.parse_and()?;

        while let Some(Ok(token)) = &self.current_token {
            match token {
                Token::Or | Token::SemiColon => {
                    self.advance();
                    let right = self.parse_and()?;
                    left = FilterExpr::Or(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_and<T>(&mut self) -> Result<FilterExpr<T>, SyntaxError>
    where
        T: FromStr,
    {
        let mut left = self.parse_unary()?;

        while let Some(Ok(Token::And)) = &self.current_token {
            self.advance();
            let right = self.parse_unary()?;
            left = FilterExpr::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_unary<T>(&mut self) -> Result<FilterExpr<T>, SyntaxError>
    where
        T: FromStr,
    {
        match &self.current_token {
            Some(Ok(Token::Not)) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(FilterExpr::Not(Box::new(expr)))
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary<T>(&mut self) -> Result<FilterExpr<T>, SyntaxError>
    where
        T: FromStr,
    {
        match &self.current_token {
            Some(Ok(Token::LParen)) => {
                self.advance();
                let expr = self.parse_or()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Some(Ok(Token::Value)) => {
                let value = self.current_slice.to_string();
                self.advance();
                Ok(FilterExpr::Raw(value))
            }
            Some(Ok(token)) => Err(SyntaxError {
                src: self.input.to_string(),
                span: (self.lexer.span().start, self.lexer.span().len()).into(),
                message: format!("Unexpected token: {:?}", token),
            }),
            Some(Err(e)) => Err(e.clone()),
            None => Err(SyntaxError {
                src: self.input.to_string(),
                span: (self.input.len(), 0).into(),
                message: "Unexpected end of input".to_string(),
            }),
        }
    }
}

pub trait Evaluator<T, V> {
    fn evaluate(&self, expr: &FilterExpr<V>, item: &T) -> bool;
}

impl<V> FilterExpr<V> {
    pub fn map<U, F>(self, f: F) -> FilterExpr<U>
    where
        F: Fn(String) -> U,
    {
        self.transform(|raw| Ok::<U, std::convert::Infallible>(f(raw)))
            .unwrap()
    }

    pub fn try_map<U, F, E>(self, f: F) -> Result<FilterExpr<U>, E>
    where
        F: Fn(String) -> Result<U, E> + Copy,
    {
        self.transform(f)
    }

    // Helper method that handles the actual transformation
    fn transform<U, F, E>(self, f: F) -> Result<FilterExpr<U>, E>
    where
        F: Fn(String) -> Result<U, E> + Copy,
    {
        match self {
            FilterExpr::And(left, right) => {
                let left = left.transform(f)?;
                let right = right.transform(f)?;
                Ok(FilterExpr::And(Box::new(left), Box::new(right)))
            }
            FilterExpr::Or(left, right) => {
                let left = left.transform(f)?;
                let right = right.transform(f)?;
                Ok(FilterExpr::Or(Box::new(left), Box::new(right)))
            }
            FilterExpr::Not(expr) => {
                let transformed = expr.transform(f)?;
                Ok(FilterExpr::Not(Box::new(transformed)))
            }
            FilterExpr::Value(_) => {
                // This should never happen as Raw variants should always be transformed first
                unreachable!("Value variant encountered during Raw transformation")
            }
            FilterExpr::Raw(raw) => Ok(FilterExpr::Value(f(raw)?)),
        }
    }
}
