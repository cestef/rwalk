use std::sync::Arc;

use crate::cli::interactive::{Command, State};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use log::error;
use rhai::{Dynamic, Engine, Scope};
use rustyline::DefaultEditor;
use tokio::sync::Mutex;

fn execute(engine: &mut Engine, scope: &mut Scope, line: String) -> Result<()> {
    let maybe_out = engine.eval_with_scope::<Dynamic>(scope, &line);
    match maybe_out {
        Ok(out) => {
            let out = out.to_string().trim().to_string();
            if out.is_empty() {
                return Ok(());
            }
            println!("{}", out);
        }
        Err(e) => {
            error!("{}", e);
        }
    }
    Ok(())
}
#[derive(Debug)]
pub struct EvalCommand;

#[async_trait]
impl Command for EvalCommand {
    fn name(&self) -> &'static str {
        "eval"
    }

    fn description(&self) -> &'static str {
        "Evaluates a script"
    }

    async fn run(
        &self,
        rl: Arc<Mutex<DefaultEditor>>,
        args: Vec<&str>,
        state: Arc<Mutex<State>>,
        engine: Arc<Mutex<Engine>>,
        scope: Arc<Mutex<Scope<'_>>>,
    ) -> Result<()> {
        let mut scope = scope.lock().await;
        let mut engine = engine.lock().await;
        let state = state.lock().await;
        if let Some(last_result) = &state.last_result {
            scope.set_or_push("tree", last_result.clone());
        }
        scope.set_or_push("opts", state.opts.clone());
        if args.is_empty() {
            // Enter interactive mode
            let mut rl = rl.lock().await;
            loop {
                let readline = rl.readline("eval> ");
                match readline {
                    Ok(mut line) => {
                        line = line.trim().to_string();
                        if line.is_empty() {
                            continue;
                        }
                        match line.as_str() {
                            "exit" | "quit" | "q" => break,
                            "clear" | "cls" => {
                                rl.clear_screen()?;
                                continue;
                            }
                            _ => {}
                        }
                        rl.add_history_entry(line.as_str())?;
                        execute(&mut engine, &mut scope, line)?;
                    }
                    Err(_) => break,
                }
            }
        } else {
            let line = args.join(" ");
            execute(&mut engine, &mut scope, line)?;
        }

        Ok(())
    }
}
