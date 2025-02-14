use num_traits::PrimInt;
use serde::Deserialize;
use std::{fmt::Display, str::FromStr};

use crate::error::{syntax_error, RwalkError, SyntaxError};

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum EngineMode {
    Recursive,
    Template,
}

impl FromStr for EngineMode {
    type Err = RwalkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "recursive" | "r" => Ok(EngineMode::Recursive),
            "template" | "t" => Ok(EngineMode::Template),
            _ => Err(syntax_error!(
                (0, s.len()),
                s,
                "Invalid engine mode: '{}', available: 'recursive', 'template'",
                s
            )),
        }
    }
}

impl Display for EngineMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineMode::Recursive => write!(f, "recursive"),
            EngineMode::Template => write!(f, "template"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct IntRange<T>
where
    T: PrimInt,
{
    pub start: T,
    pub end: T,
}

impl<T> IntRange<T>
where
    T: PrimInt,
{
    pub fn new(start: T, end: T) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, value: T) -> bool {
        value >= self.start && value <= self.end
    }
}

impl<T> std::fmt::Debug for IntRange<T>
where
    T: PrimInt + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

impl<T> FromStr for IntRange<T>
where
    T: PrimInt + FromStr + Display,
{
    type Err = RwalkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Handle empty string
        if s.is_empty() {
            return Err(syntax_error!((0, 0), s, "Empty range expression"));
        }

        let parts: Vec<&str> = s.split('-').collect();

        // Handle single value cases (>, <, or exact)
        if parts.len() == 1 {
            let first_char = parts[0].chars().next().unwrap();
            match first_char {
                '>' | '<' => {
                    let value_str = &parts[0][1..];
                    if value_str.is_empty() {
                        return Err(syntax_error!(
                            (1, 1),
                            s,
                            "Missing value after '{}' operator",
                            first_char
                        ));
                    }

                    match value_str.parse::<T>() {
                        Ok(value) => match first_char {
                            '>' => Ok(IntRange::new(value + T::one(), T::max_value())),
                            '<' => Ok(IntRange::new(T::min_value(), value - T::one())),
                            _ => unreachable!(),
                        },
                        Err(_) => Err(syntax_error!(
                            (1, value_str.len()),
                            s,
                            "Invalid numeric value: '{}'",
                            value_str
                        )),
                    }
                }
                _ => match parts[0].parse() {
                    Ok(value) => Ok(IntRange::new(value, value)),
                    Err(_) => Err(syntax_error!(
                        (0, parts[0].len()),
                        s,
                        "Invalid numeric value: '{}'",
                        parts[0]
                    )),
                },
            }
        } else if parts.len() == 2 {
            // Handle range with start and end values
            let start = match parts[0].parse() {
                Ok(v) => v,
                Err(_) => {
                    return Err(syntax_error!(
                        (0, parts[0].len()),
                        s,
                        "Invalid start value: '{}'",
                        parts[0]
                    ))
                }
            };

            let end = match parts[1].parse() {
                Ok(v) => v,
                Err(_) => {
                    return Err(syntax_error!(
                        (parts[0].len() + 1, parts[1].len()),
                        s,
                        "Invalid end value: '{}'",
                        parts[1]
                    ))
                }
            };

            if start > end {
                return Err(syntax_error!(
                    (0, s.len()),
                    s,
                    "Start value {} cannot be greater than end value {}",
                    start,
                    end
                ));
            }

            Ok(IntRange::new(start, end))
        } else {
            // Handle invalid format with too many hyphens
            Err(syntax_error!(
                (0, s.len()),
                s,
                "Invalid range format: too many hyphens"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_syntax_error(input: &str, expected_span: (usize, usize), expected_msg: &str) {
        match input.parse::<IntRange<i32>>() {
            Ok(_) => panic!("Expected error for input: {}", input),
            Err(RwalkError::SyntaxError(err)) => {
                assert_eq!((err.span.offset(), err.span.len()), expected_span);
                assert_eq!(err.message, expected_msg);
            }
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn test_parse_empty() {
        assert_syntax_error("", (0, 0), "Empty range expression");
    }

    #[test]
    fn test_parse_invalid_single() {
        assert_syntax_error("abc", (0, 3), "Invalid numeric value: 'abc'");
    }

    #[test]
    fn test_parse_invalid_start() {
        assert_syntax_error("abc-10", (0, 3), "Invalid start value: 'abc'");
    }

    #[test]
    fn test_parse_invalid_end() {
        assert_syntax_error("10-abc", (3, 6), "Invalid end value: 'abc'");
    }

    #[test]
    fn test_parse_invalid_operator() {
        assert_syntax_error(">", (1, 1), "Missing value after '>' operator");
    }

    #[test]
    fn test_parse_invalid_operator_value() {
        assert_syntax_error(">abc", (1, 4), "Invalid numeric value: 'abc'");
    }

    #[test]
    fn test_parse_invalid_range() {
        assert_syntax_error(
            "10-5",
            (0, 4),
            "Start value 10 cannot be greater than end value 5",
        );
    }

    #[test]
    fn test_parse_too_many_parts() {
        assert_syntax_error("1-2-3", (0, 5), "Invalid range format: too many hyphens");
    }

    #[test]
    fn test_valid_ranges() {
        assert!("5".parse::<IntRange<i32>>().is_ok());
        assert!("1-10".parse::<IntRange<i32>>().is_ok());
        assert!(">5".parse::<IntRange<i32>>().is_ok());
        assert!("<5".parse::<IntRange<i32>>().is_ok());
    }
}
