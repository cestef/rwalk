use super::{Command, CommandContext, CommandRegistry};
use crate::Result;
use owo_colors::OwoColorize;

#[derive(Debug)]
pub struct HelpCommand;

#[async_trait::async_trait]
impl Command<CommandContext> for HelpCommand {
    async fn execute(&self, _ctx: &mut CommandContext, _args: &str) -> Result<()> {
        println!("{}", "Available commands:".bold());
        let list = CommandRegistry::list();

        for (name, aliases) in list {
            let cmd = CommandRegistry::construct(name)?;
            let name_with_usage = format!("{} {}", name.green().bold(), cmd.usage());
            let aliases_str = if !aliases.is_empty() {
                format!("({})", aliases.join(", ")).dimmed().to_string()
            } else {
                String::new()
            };

            println!(
                "  {}{}\n    {}",
                name_with_usage,
                aliases_str,
                cmd.description(),
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

    fn description(&self) -> &'static str {
        "Display this help message"
    }

    fn construct() -> Box<dyn Command<CommandContext>>
    where
        Self: Sized + 'static,
    {
        Box::new(HelpCommand)
    }
}
