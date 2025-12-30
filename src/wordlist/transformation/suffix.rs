use super::Transform;
use crate::{
    error::{error, RwalkError},
    Result,
};

#[derive(Debug, Clone)]
pub struct SuffixTransformer {
    suffix: String,
}

impl Transform<String> for SuffixTransformer {
    fn transform(&self, item: &mut String) {
        item.push_str(&self.suffix);
    }

    fn name() -> &'static str {
        "suffix"
    }

    fn aliases() -> &'static [&'static str] {
        &["s", "suf"]
    }

    fn construct(arg: Option<&str>) -> Result<Box<dyn Transform<String>>> {
        if let Some(arg) = arg {
            Ok(Box::new(SuffixTransformer {
                suffix: arg.to_string(),
            }))
        } else {
            Err(error!("Suffix transformer needs a prefix to be applied"))
        }
    }
}
