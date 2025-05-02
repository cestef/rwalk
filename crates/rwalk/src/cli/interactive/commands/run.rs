use clap::Parser;

use super::{ArgType, Command, CommandContext};
use crate::{Result, run};

#[derive(Debug)]
pub struct RunCommand;

#[async_trait::async_trait]
impl<'a> Command<CommandContext<'a>> for RunCommand {
    async fn execute(&self, ctx: &mut CommandContext, args: &str) -> Result<()> {
        let mut opts = ctx.opts.clone();
        if !args.is_empty() {
            let args = std::iter::once("rwalk".to_string())
                .chain(std::iter::once(
                    opts.url.as_ref().map(|u| u.to_string()).unwrap_or_default(),
                ))
                .chain(
                    opts.wordlists
                        .iter()
                        .map(|w| format!("{}:{}", w.0, w.1))
                        .collect::<Vec<_>>()
                        .into_iter(),
                )
                .chain(args.split_whitespace().map(|s| s.to_string()));
            opts.try_update_from(args)?;
        }
        let mut scope = ctx.scope.lock().await;
        run(opts, Some(&mut scope)).await?;
        Ok(())
    }

    fn name() -> &'static str {
        "run"
    }

    fn aliases() -> &'static [&'static str] {
        &["r", "start"]
    }

    fn description(&self) -> &'static str {
        "Run rwalk with the current options"
    }

    fn construct() -> Box<dyn Command<CommandContext<'a>>>
    where
        Self: Sized + 'static,
    {
        Box::new(RunCommand)
    }

    fn args(&self) -> Option<&'static [ArgType]> {
        Some(&[ArgType::Url, ArgType::Path])
    }
}
