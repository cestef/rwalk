use super::{Command, CommandContext};
use crate::Result;

#[derive(Debug)]
pub struct ClearCommand;

#[async_trait::async_trait]
impl Command<CommandContext> for ClearCommand {
    async fn execute(&self, ctx: &mut CommandContext, _args: &str) -> Result<()> {
        ctx.editor.lock().await.clear_screen()?;
        Ok(())
    }

    fn name() -> &'static str {
        "clear"
    }

    fn aliases() -> &'static [&'static str] {
        &["c", "cls"]
    }

    fn description(&self) -> &'static str {
        "Clear the screen"
    }

    fn construct() -> Box<dyn Command<CommandContext>>
    where
        Self: Sized + 'static,
    {
        Box::new(ClearCommand)
    }
}
