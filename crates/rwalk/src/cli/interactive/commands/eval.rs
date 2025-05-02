use super::{Command, CommandContext};
use crate::{Result, print_error};
use owo_colors::OwoColorize;
use rhai::Dynamic;

#[derive(Debug)]
pub struct EvalCommand;

#[async_trait::async_trait]
impl<'a> Command<CommandContext<'a>> for EvalCommand {
    async fn execute(&self, ctx: &mut CommandContext, args: &str) -> Result<()> {
        let args = args.trim();

        if args.is_empty() {
            loop {
                let maybe_line;
                {
                    maybe_line = ctx.editor.lock().await.readline("rwalk (eval)> ");
                }

                if matches!(
                    maybe_line,
                    Err(rustyline::error::ReadlineError::Interrupted
                        | rustyline::error::ReadlineError::Eof)
                ) {
                    break;
                }
                let line = maybe_line?;
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let mut scope = ctx.scope.lock().await;
                let maybe_res = ctx.engine.eval_with_scope::<Dynamic>(&mut scope, line);

                match maybe_res {
                    Ok(res) => println!("{}", res),
                    Err(e) => print_error!("Error: {}", e),
                }
            }
        } else {
            let mut scope = ctx.scope.lock().await;
            let maybe_res = ctx.engine.eval_with_scope::<Dynamic>(&mut scope, args);
            match maybe_res {
                Ok(res) => println!("{}", res),
                Err(e) => {
                    print_error!("Error: {}", e);
                }
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
