use super::{Command, CommandContext};
use crate::{Result, RwalkError};

#[derive(Debug)]
pub struct AppendCommand;

#[async_trait::async_trait]
impl Command<CommandContext> for AppendCommand {
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
        if let Some(mut current_value) = ctx.opts.get_path(&field) {
            if let Some(array) = current_value.as_array_mut() {
                array.push(parsed_value);
                ctx.opts
                    .set_path(&field, current_value)
                    .map_err(|e| RwalkError::InvalidValue(e.to_string()))?;
            } else {
                return Err(RwalkError::InvalidValue(
                    "Field is not an array".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn name() -> &'static str {
        "append"
    }

    fn aliases() -> &'static [&'static str] {
        &["a", "add", "push"]
    }

    fn args(&self) -> Option<&'static [super::ArgType]> {
        Some(&[super::ArgType::OptionField, super::ArgType::Any])
    }

    fn help(&self) -> &'static str {
        "Append a value to an array field in the current context. Usage: append <field> <value>"
    }

    fn construct() -> Box<dyn Command<CommandContext>>
    where
        Self: Sized + 'static,
    {
        Box::new(AppendCommand)
    }
}
