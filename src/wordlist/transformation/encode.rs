use std::str::FromStr;

use base64::Engine;

use super::Transform;
use crate::{
    error::{error, RwalkError},
    Result,
};

#[derive(Debug, Clone)]
pub struct EncodeTransformer {
    format: EncodeFormat,
}

#[derive(Debug, Clone)]
pub enum EncodeFormat {
    Url,
    Base64,
    Hex,
}

impl FromStr for EncodeFormat {
    type Err = RwalkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "url" | "u" => Ok(EncodeFormat::Url),
            "base64" | "b64" => Ok(EncodeFormat::Base64),
            "hex" | "h" => Ok(EncodeFormat::Hex),
            _ => Err(error!("Invalid encode format: {}", s)),
        }
    }
}

impl Transform<String> for EncodeTransformer {
    fn transform(&self, item: &mut String) {
        match self.format {
            EncodeFormat::Url => *item = urlencoding::encode(&item).into_owned(),
            EncodeFormat::Base64 => *item = base64::engine::general_purpose::STANDARD.encode(&item),
            EncodeFormat::Hex => *item = hex::encode(&item),
        }
    }

    fn name() -> &'static str {
        "encode"
    }

    fn aliases() -> &'static [&'static str] {
        &["e", "enc"]
    }

    fn construct(arg: Option<&str>) -> Result<Box<dyn Transform<String>>> {
        if let Some(arg) = arg {
            Ok(Box::new(EncodeTransformer {
                format: arg.parse()?,
            }))
        } else {
            Err(error!("Suffix transformer needs a prefix to be applied"))
        }
    }
}
