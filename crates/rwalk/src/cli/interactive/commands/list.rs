use owo_colors::OwoColorize;

use super::{Command, CommandContext};
use crate::Result;

#[derive(Debug)]
pub struct ListCommand;
#[async_trait::async_trait]
impl Command<CommandContext> for ListCommand {
    async fn execute(&self, ctx: &mut CommandContext, _args: &str) -> Result<()> {
        let fields = ctx.opts.as_nested_map();
        let max_key_len = fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
        for (key, value) in fields {
            println!(
                "{} {dots} = {}",
                key.bold(),
                serde_json::to_string(&value)?.green(),
                dots = "·".repeat(max_key_len - key.len()).dimmed(),
            );
        }
        Ok(())
    }

    fn name() -> &'static str {
        "list"
    }

    fn aliases() -> &'static [&'static str] {
        &["options", "ls"]
    }

    fn help(&self) -> &'static str {
        "List current options"
    }

    fn construct() -> Box<dyn Command<CommandContext>>
    where
        Self: Sized + 'static,
    {
        Box::new(ListCommand)
    }
}
