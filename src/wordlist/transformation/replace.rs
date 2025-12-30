use super::Transform;
use crate::{
    cli::parse::parse_keyval_with_sep,
    error::{error, RwalkError},
    Result,
};

#[derive(Debug, Clone)]
pub struct ReplaceTransformer {
    replace: (String, String),
}

fn replace_in_place(s: &mut String, from: &str, to: &str) {
    let mut start = 0;
    while let Some(pos) = s[start..].find(from) {
        let pos = pos + start;
        s.replace_range(pos..pos + from.len(), to);
        start = pos + to.len();
    }
}

impl Transform<String> for ReplaceTransformer {
    fn transform(&self, item: &mut String) {
        replace_in_place(item, &self.replace.0, &self.replace.1);
    }

    fn name() -> &'static str {
        "replace"
    }

    fn aliases() -> &'static [&'static str] {
        &["rp", "sub"]
    }

    fn construct(arg: Option<&str>) -> Result<Box<dyn Transform<String>>> {
        if let Some(arg) = arg {
            Ok(Box::new(ReplaceTransformer {
                replace: parse_keyval_with_sep(arg, '=')?,
            }))
        } else {
            Err(error!(
                "Replace transformer needs a key-value pair to be applied"
            ))
        }
    }
}
