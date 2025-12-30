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
            let mut args_vec = vec!["rwalk".to_string()];
            if let Some(ref url) = opts.url {
                args_vec.push(url.to_string());
            }
            if !opts.wordlists.is_empty() {
                args_vec.push(
                    opts.wordlists
                        .iter()
                        .map(|w| format!("{}:{}", w.0, w.1))
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }

            args_vec.extend(args.split_whitespace().map(|s| s.to_string()));
            opts.try_update_from(args_vec.into_iter())?;
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
