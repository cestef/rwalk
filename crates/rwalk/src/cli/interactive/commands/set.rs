use super::{Command, CommandContext};
use crate::{Result, RwalkError};

#[derive(Debug)]
pub struct SetCommand;

#[async_trait::async_trait]
impl<'a> Command<CommandContext<'a>> for SetCommand {
    async fn execute(&self, ctx: &mut CommandContext, args: &str) -> Result<()> {
        let (field, value) = if args.is_empty() {
            return Err(RwalkError::InvalidCommand(
                "No arguments provided".to_string(),
            ));
        } else {
            let args = args.split_whitespace().collect::<Vec<_>>();
            if args.len() < 2 {
                return Err(RwalkError::InvalidCommand(
                    "Not enough arguments provided".to_string(),
                ));
            }
            (args[0].to_string(), args[1..].join(" "))
        };
        let parsed_value = match serde_json::from_str::<serde_json::Value>(&value) {
            Ok(json_value) => json_value,
            Err(_) => serde_json::Value::String(value.to_string()),
        };

        ctx.opts
            .set_path(&field, parsed_value)
            .map_err(|e| RwalkError::InvalidValue(e.to_string()))?;

        Ok(())
    }

    fn name() -> &'static str {
        "set"
    }

    fn aliases() -> &'static [&'static str] {
        &["s"]
    }

    fn args(&self) -> Option<&'static [super::ArgType]> {
        Some(&[super::ArgType::OptionField, super::ArgType::Any])
    }

    fn description(&self) -> &'static str {
        "Set a field's value"
    }

    fn construct() -> Box<dyn Command<CommandContext<'a>>>
    where
        Self: Sized + 'static,
    {
        Box::new(SetCommand)
    }
}
