use super::Transform;
use crate::{
    error::{error, RwalkError},
    Result,
};

#[derive(Debug, Clone)]
pub struct RemoveTransformer {
    pattern: String,
}

impl Transform<String> for RemoveTransformer {
    fn transform(&self, item: &mut String) {
        item.retain(|c| !self.pattern.contains(c));
    }

    fn name() -> &'static str {
        "prefix"
    }

    fn aliases() -> &'static [&'static str] {
        &["p", "pre"]
    }

    fn construct(arg: Option<&str>) -> Result<Box<dyn Transform<String>>> {
        if let Some(arg) = arg {
            Ok(Box::new(RemoveTransformer {
                pattern: arg.to_string(),
            }))
        } else {
            Err(error!("Remove transformer needs a pattern to be applied"))
        }
    }
}
