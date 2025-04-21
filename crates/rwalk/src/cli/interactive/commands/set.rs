use super::{Command, CommandContext};
use crate::{Result, RwalkError};

#[derive(Debug)]
pub struct SetCommand;

#[async_trait::async_trait]
impl Command<CommandContext> for SetCommand {
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

        ctx.opts
            .set_path(
                &field,
                serde_json::from_str(&value).map_err(|e| {
                    RwalkError::InvalidValue(format!("Failed to parse value: {}", e))
                })?,
            )
            .map_err(|e| RwalkError::InvalidValue(e.to_string()))?;

        Ok(())
    }

    fn name() -> &'static str {
        "set"
    }

    fn aliases() -> &'static [&'static str] {
        &["s"]
    }

    fn help(&self) -> &'static str {
        "Set a field in the current context. Usage: set <field> <value>"
    }

    fn construct() -> Box<dyn Command<CommandContext>>
    where
        Self: Sized + 'static,
    {
        Box::new(SetCommand)
    }
}
