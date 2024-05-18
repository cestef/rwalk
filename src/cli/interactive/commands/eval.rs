use crate::cli::interactive::State;
use color_eyre::eyre::Result;
use log::error;
use rhai::{Dynamic, Engine, Scope};
use rustyline::DefaultEditor;

pub fn eval(
    rl: &mut DefaultEditor,
    args: Vec<&str>,
    state: &mut State,
    engine: &mut Engine,
    scope: &mut Scope<'_>,
) -> Result<()> {
    if let Some(last_result) = &state.last_result {
        scope.set_or_push("tree", last_result.clone());
    }
    scope.set_or_push("opts", state.opts.clone());
    if args.is_empty() {
        // Enter interactive mode
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
                    execute(engine, scope, line)?;
                }
                Err(_) => break,
            }
        }
    } else {
        let line = args.join(" ");
        execute(engine, scope, line)?;
    }

    Ok(())
}

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
