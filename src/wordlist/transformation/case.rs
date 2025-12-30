use std::str::FromStr;

use super::Transform;
use crate::{
    Result,
    error::{RwalkError, error},
};

#[derive(Debug, Clone)]
pub enum Case {
    Upper,
    Lower,
    Capitalize,
}

impl FromStr for Case {
    type Err = RwalkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "upper" | "up" | "u" => Ok(Case::Upper),
            "lower" | "low" | "l" => Ok(Case::Lower),
            "title" | "capitalize" | "cap" | "c" => Ok(Case::Capitalize),
            _ => Err(error!("Invalid case: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CaseTransformer {
    case: Case,
}

impl Transform<String> for CaseTransformer {
    fn transform(&self, item: &mut String) {
        match self.case {
            Case::Upper => item.make_ascii_uppercase(),
            Case::Lower => item.make_ascii_lowercase(),
            Case::Capitalize => {
                if let Some(first) = item.chars().next() {
                    item.replace_range(..1, &first.to_uppercase().to_string());
                }
            }
        }
    }

    fn name() -> &'static str {
        "case"
    }

    fn aliases() -> &'static [&'static str] {
        &["c"]
    }

    fn construct(arg: Option<&str>) -> Result<Box<dyn Transform<String>>> {
        if let Some(arg) = arg {
            Ok(Box::new(CaseTransformer { case: arg.parse()? }))
        } else {
            Err(error!(
                "Case transformer needs to be either 'upper' or 'lower'"
            ))
        }
    }
}
