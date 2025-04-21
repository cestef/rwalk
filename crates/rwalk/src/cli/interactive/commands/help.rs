use super::{Command, CommandContext, CommandRegistry};
use crate::Result;
use owo_colors::OwoColorize;

#[derive(Debug)]
pub struct HelpCommand {}

#[async_trait::async_trait]
impl Command<CommandContext> for HelpCommand {
    async fn execute(&self, _ctx: &mut CommandContext, _args: &str) -> Result<()> {
        println!("{}", "Available commands:".bold());
        for (name, aliases) in CommandRegistry::list() {
            let cmd = CommandRegistry::construct(name)?;
            println!(
                "  {} {}: {}",
                name.green().bold(),
                if !aliases.is_empty() {
                    format!("({})", aliases.join(", ")).dimmed().to_string()
                } else {
                    String::new()
                },
                cmd.help().italic()
            );
        }
        Ok(())
    }

    fn name() -> &'static str {
        "help"
    }

    fn aliases() -> &'static [&'static str] {
        &["h", "?"]
    }

    fn help(&self) -> &'static str {
        "Display this help message"
    }

    fn construct() -> Box<dyn Command<CommandContext>>
    where
        Self: Sized + 'static,
    {
        Box::new(HelpCommand {})
    }
}
