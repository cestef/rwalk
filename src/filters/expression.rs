use crate::error::SyntaxError;
use crate::Result;
use std::fmt::Debug;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
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
    #[token("&")]
    And,

    #[token("|")]
    Or,

    #[token(";")]
    SemiColon,

    #[token("!")]
    Not,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    // 1. Either a sequence of non-special chars (not including whitespace)
    // 2. Or an escape sequence followed by any char (including whitespace)
    // This prevents overlap with the skip pattern
    #[regex(r"([^&|;!()\\\s]+|\\[\s&|;!()\\])+")]
    Value,
}

// Helper function to unescape a string
fn unescape_string(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next_char) = chars.next() {
                // Keep the actual character after the escape
                result.push(next_char);
            }
        } else {
            result.push(ch);
        }
    }

    result
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
                // Unescape the value before storing it
                let unescaped_value = unescape_string(self.current_slice);
                self.advance();
                Ok(FilterExpr::Raw(unescaped_value))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expression() {
        let mut parser = ExprParser::new("value");
        let expr = parser.parse::<String>().unwrap();

        match expr {
            FilterExpr::Raw(value) => {
                assert_eq!(value, "value");
            }
            _ => panic!("Expected Raw variant"),
        }
    }

    #[test]
    fn test_escaped_values() {
        let mut parser = ExprParser::new(r"value\&with\|escaped\;chars");
        let expr = parser.parse::<String>().unwrap();

        match expr {
            FilterExpr::Raw(value) => {
                assert_eq!(value, "value&with|escaped;chars");
            }
            _ => panic!("Expected Raw variant"),
        }
    }

    #[test]
    fn test_escaped_whitespace() {
        let mut parser = ExprParser::new(r"prefix\ suffix");
        let expr = parser.parse::<String>().unwrap();

        match expr {
            FilterExpr::Raw(value) => {
                assert_eq!(value, "prefix suffix");
            }
            _ => panic!("Expected Raw variant"),
        }
    }

    #[test]
    fn test_complex_expression_with_escapes() {
        let input = r"normal & escaped\&value | another\|value";
        let mut parser = ExprParser::new(input);
        let expr = parser.parse::<String>().unwrap();

        assert_eq!(
            expr,
            FilterExpr::Or(
                Box::new(FilterExpr::And(
                    Box::new(FilterExpr::Raw("normal".to_string())),
                    Box::new(FilterExpr::Raw("escaped&value".to_string()))
                )),
                Box::new(FilterExpr::Raw("another|value".to_string()))
            )
        );
    }

    #[test]
    fn test_map() {
        let input = r"normal & escaped\&value | another\|value";
        let mut parser = ExprParser::new(input);
        let expr = parser.parse::<String>().unwrap();

        let mapped = expr.map(|s| s.to_uppercase());
        assert_eq!(
            mapped,
            FilterExpr::Or(
                Box::new(FilterExpr::And(
                    Box::new(FilterExpr::Value("NORMAL".to_string())),
                    Box::new(FilterExpr::Value("ESCAPED&VALUE".to_string()))
                )),
                Box::new(FilterExpr::Value("ANOTHER|VALUE".to_string()))
            )
        );
    }

    #[test]
    fn test_try_map() {
        let input = r"normal & escaped\&value | another\|value";
        let mut parser = ExprParser::new(input);
        let expr = parser.parse::<String>().unwrap();

        let mapped = expr
            .try_map(|s| Ok::<String, String>(s.to_uppercase()))
            .unwrap();
        assert_eq!(
            mapped,
            FilterExpr::Or(
                Box::new(FilterExpr::And(
                    Box::new(FilterExpr::Value("NORMAL".to_string())),
                    Box::new(FilterExpr::Value("ESCAPED&VALUE".to_string()))
                )),
                Box::new(FilterExpr::Value("ANOTHER|VALUE".to_string()))
            )
        );
    }

    #[test]
    fn test_missing_parentheses() {
        let input = "value & (value | value";
        let mut parser = ExprParser::new(input);
        let result = parser.parse::<String>();

        assert!(result.is_err());
    }

    #[test]
    fn test_double_parentheses() {
        let input = "value & ((value | value))";
        let mut parser = ExprParser::new(input);
        let result = parser.parse::<String>();

        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_parentheses() {
        let input = "value & ()";
        let mut parser = ExprParser::new(input);
        let result = parser.parse::<String>();

        assert!(result.is_err());
    }

    #[test]
    fn test_unexpected_token() {
        let input = "value & | value";
        let mut parser = ExprParser::new(input);
        let result = parser.parse::<String>();

        assert!(result.is_err());
    }

    #[test]
    fn test_unexpected_end_of_input() {
        let input = "value &";
        let mut parser = ExprParser::new(input);
        let result = parser.parse::<String>();

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_token() {
        let input = "value & value $ value";
        let mut parser = ExprParser::new(input);
        let result = parser.parse::<String>();
        println!("{:?}", result);
        assert!(result.is_err());
    }
}
