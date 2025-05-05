use super::{Command, CommandContext};
use crate::{Result, print_error};
use owo_colors::OwoColorize;
use rhai::Dynamic;
use std::path::Path;
use tokio::fs;

#[derive(Debug)]
pub struct EvalCommand;

#[async_trait::async_trait]
impl<'a> Command<CommandContext<'a>> for EvalCommand {
    async fn execute(&self, ctx: &mut CommandContext, args: &str) -> Result<()> {
        let args = args.trim();

        // Check if we're evaluating a file
        if let Some(file_path) = args.strip_prefix('@') {
            let path = Path::new(file_path);
            if !path.exists() {
                print_error!("File not found: {}", file_path);
                return Ok(());
            }

            let script = fs::read_to_string(path).await?;
            let mut scope = ctx.scope.lock().await;
            return match ctx.engine.eval_with_scope::<Dynamic>(&mut scope, &script) {
                Ok(res) => {
                    println!("{}", res);
                    Ok(())
                }
                Err(e) => {
                    print_error!("Error: {}", e);
                    Ok(())
                }
            };
        }

        if args.is_empty() {
            // Interactive eval mode
            let mut editor = ctx.editor.lock().await;

            loop {
                let line = match editor.readline("rwalk (eval)> ") {
                    Ok(line) => line,
                    Err(
                        rustyline::error::ReadlineError::Interrupted
                        | rustyline::error::ReadlineError::Eof,
                    ) => break,
                    Err(e) => return Err(e.into()),
                };

                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let mut scope = ctx.scope.lock().await;
                match ctx.engine.eval_with_scope::<Dynamic>(&mut scope, line) {
                    Ok(res) => println!("{}", res),
                    Err(e) => print_error!("Error: {}", e),
                }
            }
        } else {
            // Single expression evaluation
            let mut scope = ctx.scope.lock().await;
            match ctx.engine.eval_with_scope::<Dynamic>(&mut scope, args) {
                Ok(res) => println!("{}", res),
                Err(e) => print_error!("Error: {}", e),
            }
        }

        Ok(())
    }

    fn name() -> &'static str {
        "eval"
    }

    fn aliases() -> &'static [&'static str] {
        &["e"]
    }

    fn args(&self) -> Option<&'static [super::ArgType]> {
        Some(&[super::ArgType::Any])
    }

    fn description(&self) -> &'static str {
        "Evaluate an expression in the current context"
    }

    fn construct() -> Box<dyn Command<CommandContext<'a>>>
    where
        Self: Sized + 'static,
    {
        Box::new(EvalCommand)
    }
}
