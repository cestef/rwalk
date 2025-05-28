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
            // Define a state struct to track buffer and delimiters
            struct EvalState {
                buffer: String,
                braces: usize,
                parens: usize,
                brackets: usize,
            }

            let mut state = EvalState {
                buffer: String::new(),
                braces: 0,
                parens: 0,
                brackets: 0,
            };

            loop {
                // Determine prompt based on buffer state
                let prompt = if state.buffer.is_empty() {
                    "rwalk (eval)> "
                } else if state.braces > 0 || state.parens > 0 || state.brackets > 0 {
                    "... "
                } else {
                    "... "
                };

                let line = match editor.readline(prompt) {
                    Ok(line) => line,
                    Err(
                        rustyline::error::ReadlineError::Interrupted
                        | rustyline::error::ReadlineError::Eof,
                    ) => break,
                    Err(e) => return Err(e.into()),
                };

                let line = line.trim();

                // Empty line executes the current buffer if not in a block
                let in_block = state.braces > 0 || state.parens > 0 || state.brackets > 0;
                if line.is_empty() && !state.buffer.is_empty() && !in_block {
                    let mut scope = ctx.scope.lock().await;
                    match ctx
                        .engine
                        .eval_with_scope::<Dynamic>(&mut scope, &state.buffer)
                    {
                        Ok(res) => println!("{}", res),
                        Err(e) => print_error!("Error: {}", e),
                    }

                    state.buffer.clear();
                    continue;
                }

                // Update block state by counting delimiters
                for c in line.chars() {
                    match c {
                        '{' => state.braces += 1,
                        '}' => state.braces = state.braces.saturating_sub(1),
                        '(' => state.parens += 1,
                        ')' => state.parens = state.parens.saturating_sub(1),
                        '[' => state.brackets += 1,
                        ']' => state.brackets = state.brackets.saturating_sub(1),
                        _ => {}
                    }
                }

                // Add line to buffer
                if !state.buffer.is_empty() {
                    state.buffer.push('\n');
                }
                state.buffer.push_str(line);
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
