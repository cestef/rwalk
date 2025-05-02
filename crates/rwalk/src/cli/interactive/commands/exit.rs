use super::{Command, CommandContext};
use crate::Result;

#[derive(Debug)]
pub struct ExitCommand;

#[async_trait::async_trait]
impl Command<CommandContext> for ExitCommand {
    async fn execute(&self, ctx: &mut CommandContext, _args: &str) -> Result<()> {
        ctx.exit = true;
        println!("Goodbye \\o");
        Ok(())
    }

    fn name() -> &'static str {
        "exit"
    }

    fn aliases() -> &'static [&'static str] {
        &["quit", "q"]
    }

    fn description(&self) -> &'static str {
        "Exit the interactive shell"
    }

    fn construct() -> Box<dyn Command<CommandContext>>
    where
        Self: Sized + 'static,
    {
        Box::new(ExitCommand)
    }
}
