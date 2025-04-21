use super::Transform;
use crate::{
    error::{error, RwalkError},
    Result,
};

#[derive(Debug, Clone)]
pub struct PrefixTransformer {
    prefix: String,
}

impl Transform<String> for PrefixTransformer {
    fn transform(&self, item: &mut String) {
        item.insert_str(0, &self.prefix);
    }

    fn name() -> &'static str {
        "prefix"
    }

    fn aliases() -> &'static [&'static str] {
        &["p", "pre"]
    }

    fn construct(arg: Option<&str>) -> Result<Box<dyn Transform<String>>> {
        if let Some(arg) = arg {
            Ok(Box::new(PrefixTransformer {
                prefix: arg.to_string(),
            }))
        } else {
            Err(error!("Prefix transformer needs a prefix to be applied"))
        }
    }
}
