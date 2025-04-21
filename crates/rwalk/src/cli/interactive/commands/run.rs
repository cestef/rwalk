use clap::Parser;
use merge::Merge;

use super::{Command, CommandContext};
use crate::{Result, cli::Opts, run};

#[derive(Debug)]
pub struct RunCommand;

#[async_trait::async_trait]
impl Command<CommandContext> for RunCommand {
    async fn execute(&self, ctx: &mut CommandContext, args: &str) -> Result<()> {
        let mut opts = ctx.opts.clone();
        if !args.is_empty() {
            let args = args.split_whitespace().collect::<Vec<_>>();
            opts.merge(Opts::parse_from(["rwalk"].iter().chain(args.iter())));
        }

        run(opts).await?;
        Ok(())
    }

    fn name() -> &'static str {
        "run"
    }

    fn aliases() -> &'static [&'static str] {
        &["r", "start"]
    }

    fn help(&self) -> &'static str {
        "Run rwalk with current options (optionally specify a path)"
    }

    fn construct() -> Box<dyn Command<CommandContext>>
    where
        Self: Sized + 'static,
    {
        Box::new(RunCommand)
    }
}
