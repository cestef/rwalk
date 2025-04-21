use owo_colors::OwoColorize;

use super::{Command, CommandContext};
use crate::{Result, RwalkError};

#[derive(Debug)]
pub struct GetCommand;

#[async_trait::async_trait]
impl Command<CommandContext> for GetCommand {
    async fn execute(&self, ctx: &mut CommandContext, args: &str) -> Result<()> {
        let field = args.trim();
        if field.is_empty() {
            return Err(RwalkError::InvalidCommand(
                "Field name cannot be empty".into(),
            ));
        }

        let value = ctx.opts.get_path(&field);

        match value {
            Some(v) => {
                println!(
                    "{} = {}",
                    field.dimmed(),
                    serde_json::to_string_pretty(&v)?.green()
                );
            }
            None => {
                println!("Field '{}' not found in the current context", field);
            }
        }

        Ok(())
    }

    fn name() -> &'static str {
        "get"
    }

    fn aliases() -> &'static [&'static str] {
        &["s"]
    }

    fn help(&self) -> &'static str {
        "Get a field in the current context. Usage: get <field>"
    }

    fn construct() -> Box<dyn Command<CommandContext>>
    where
        Self: Sized + 'static,
    {
        Box::new(GetCommand)
    }
}
