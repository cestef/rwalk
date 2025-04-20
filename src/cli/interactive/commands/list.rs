use super::{Command, CommandContext};
use crate::Result;

#[derive(Debug)]
pub struct ListCommand {}
#[async_trait::async_trait]
impl Command<CommandContext> for ListCommand {
    async fn execute(&self, ctx: &mut CommandContext, _args: &str) -> Result<()> {
        println!("Current options:");
        println!("{:#?}", ctx.opts);
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
        Box::new(ListCommand {})
    }
}
