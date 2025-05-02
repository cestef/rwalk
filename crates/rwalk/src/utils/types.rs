use clap::ValueEnum;
use num_traits::PrimInt;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

use crate::{
    error::{RwalkError, SyntaxError, syntax_error},
    utils::format::color_for_status_code,
};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum ListType {
    Filters,
    Transforms,
    All,
}

impl ValueEnum for ListType {
    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            ListType::Filters => {
                Some(clap::builder::PossibleValue::new("filters").aliases(["filter", "f"]))
            }
            ListType::Transforms => {
                Some(clap::builder::PossibleValue::new("transforms").aliases(["transform", "t"]))
            }
            ListType::All => Some(clap::builder::PossibleValue::new("all")),
        }
    }

    fn value_variants<'a>() -> &'a [Self] {
        &[ListType::Filters, ListType::Transforms, ListType::All]
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EngineMode {
    Recursive,
    Template,
}

impl ValueEnum for EngineMode {
    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            EngineMode::Recursive => Some(
                clap::builder::PossibleValue::new("recursive")
                    .aliases(["r"])
                    .help("Recursively fuzz the target, increasing the depth with each request"),
            ),
            EngineMode::Template => {
                Some(clap::builder::PossibleValue::new("template").aliases(["t"]).help(
                    "Use a template to generate payloads, replacing placeholders with wordlist values",
                ))
            }
        }
    }

    fn value_variants<'a>() -> &'a [Self] {
        &[EngineMode::Recursive, EngineMode::Template]
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum HTTPMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    TRACE,
}

impl From<HTTPMethod> for reqwest::Method {
    fn from(val: HTTPMethod) -> Self {
        match val {
            HTTPMethod::GET => reqwest::Method::GET,
            HTTPMethod::POST => reqwest::Method::POST,
            HTTPMethod::PUT => reqwest::Method::PUT,
            HTTPMethod::DELETE => reqwest::Method::DELETE,
            HTTPMethod::PATCH => reqwest::Method::PATCH,
            HTTPMethod::HEAD => reqwest::Method::HEAD,
            HTTPMethod::OPTIONS => reqwest::Method::OPTIONS,
            HTTPMethod::TRACE => reqwest::Method::TRACE,
        }
    }
}

impl ValueEnum for HTTPMethod {
    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            HTTPMethod::GET => Some(clap::builder::PossibleValue::new("GET")),
            HTTPMethod::POST => Some(clap::builder::PossibleValue::new("POST")),
            HTTPMethod::PUT => Some(clap::builder::PossibleValue::new("PUT")),
            HTTPMethod::DELETE => Some(clap::builder::PossibleValue::new("DELETE")),
            HTTPMethod::PATCH => Some(clap::builder::PossibleValue::new("PATCH")),
            HTTPMethod::HEAD => Some(clap::builder::PossibleValue::new("HEAD")),
            HTTPMethod::OPTIONS => Some(clap::builder::PossibleValue::new("OPTIONS")),
            HTTPMethod::TRACE => Some(clap::builder::PossibleValue::new("TRACE")),
        }
    }

    fn value_variants<'a>() -> &'a [Self] {
        &[
            HTTPMethod::GET,
            HTTPMethod::POST,
            HTTPMethod::PUT,
            HTTPMethod::DELETE,
            HTTPMethod::PATCH,
            HTTPMethod::HEAD,
            HTTPMethod::OPTIONS,
            HTTPMethod::TRACE,
        ]
    }
}
#[derive(Clone, Copy, Deserialize, Serialize)]
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

fn range_to_string<T>(range: &IntRange<T>, format_callback: Option<&dyn Fn(&T) -> String>) -> String
where
    T: PrimInt + Display,
{
    match format_callback {
        Some(callback) => format_range(range, Some(callback)),
        None => format_range(range, None),
    }
}

fn format_range<T>(range: &IntRange<T>, format_callback: Option<&dyn Fn(&T) -> String>) -> String
where
    T: PrimInt + Display,
{
    let format_value = |value: &T| match format_callback {
        Some(callback) => callback(value),
        None => value.to_string(),
    };

    if range.start == range.end {
        format_value(&range.start)
    } else if range.start == T::min_value() {
        format!("<{}", format_value(&range.end))
    } else if range.end == T::max_value() {
        format!(">{}", format_value(&range.start))
    } else {
        format!(
            "{}-{}",
            format_value(&range.start),
            format_value(&range.end)
        )
    }
}

impl std::fmt::Debug for IntRange<u16> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = range_to_string(self, Some(&|v| color_for_status_code(&v.to_string(), *v)));
        write!(f, "{}", result)
    }
}
impl std::fmt::Debug for IntRange<usize> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", range_to_string(self, None))
    }
}
impl std::fmt::Debug for IntRange<u64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", range_to_string(self, None))
    }
}

impl<T> FromStr for IntRange<T>
where
    T: PrimInt + FromStr + Display,
{
    type Err = RwalkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_with_mapper::<fn(&str) -> Result<T, RwalkError>>(s, None)
    }
}

impl<T> IntRange<T>
where
    T: PrimInt + FromStr + Display,
{
    /// Parse a string into an IntRange with an optional mapper function
    /// The mapper function can transform the string before parsing it into a numeric type
    pub fn from_str_with_mapper<F>(s: &str, mapper: Option<F>) -> Result<Self, RwalkError>
    where
        F: Fn(&str) -> Result<T, RwalkError>,
    {
        // Handle empty string
        if s.is_empty() {
            return Err(syntax_error!((0, 0), s, "Empty range expression"));
        }

        let parts: Vec<&str> = s.split('-').collect();

        // Function to parse a value using mapper or direct parsing
        let parse_value = |value_str: &str, pos: (usize, usize)| -> Result<T, RwalkError> {
            match &mapper {
                Some(map_fn) => map_fn(value_str),
                None => match value_str.parse::<T>() {
                    Ok(v) => Ok(v),
                    Err(_) => Err(syntax_error!(
                        pos,
                        s,
                        "Invalid numeric value: '{}'",
                        value_str
                    )),
                },
            }
        };

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

                    let value = parse_value(value_str, (1, value_str.len()))?;
                    match first_char {
                        '>' => Ok(IntRange::new(value + T::one(), T::max_value())),
                        '<' => Ok(IntRange::new(T::min_value(), value - T::one())),
                        _ => unreachable!(),
                    }
                }
                _ => {
                    let value = parse_value(parts[0], (0, parts[0].len()))?;
                    Ok(IntRange::new(value, value))
                }
            }
        } else if parts.len() == 2 {
            // Handle range with start and end values
            let start = parse_value(parts[0], (0, parts[0].len()))?;
            let end = parse_value(parts[1], (parts[0].len() + 1, parts[1].len()))?;

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
        assert_syntax_error("10-abc", (3, 3), "Invalid end value: 'abc'");
    }

    #[test]
    fn test_parse_invalid_operator() {
        assert_syntax_error(">", (1, 1), "Missing value after '>' operator");
    }

    #[test]
    fn test_parse_invalid_operator_value() {
        assert_syntax_error(">abc", (1, 3), "Invalid numeric value: 'abc'");
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
