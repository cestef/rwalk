use std::sync::Arc;

use super::Opts;
use crate::{Result, print_error};
use commands::{CommandContext, CommandRegistry};
use helper::RwalkHelper;
use owo_colors::OwoColorize;
use rustyline::{Editor, config::Configurer};
use tokio::sync::Mutex;
mod commands;
mod helper;

pub async fn run(opts: Opts) -> Result<()> {
    let mut editor = Editor::new()?;
    editor.set_helper(Some(RwalkHelper));
    editor.set_auto_add_history(true);
    let editor = Arc::new(Mutex::new(editor));
    let mut ctx = CommandContext {
        exit: false,
        opts,
        editor: editor.clone(),
    };

    println!("Welcome to rwalk interactive mode! Type 'help' for available commands.");

    while !ctx.exit {
        let maybe_line;
        {
            maybe_line = editor.lock().await.readline("rwalk> ");
        }

        if matches!(
            maybe_line,
            Err(rustyline::error::ReadlineError::Interrupted | rustyline::error::ReadlineError::Eof)
        ) {
            break;
        }

        let line = maybe_line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let (command, args) = match line.find(' ') {
            Some(pos) => (&line[..pos], line[pos + 1..].trim()),
            None => (line, ""),
        };

        match CommandRegistry::construct(command) {
            Ok(cmd) => match cmd.execute(&mut ctx, args).await {
                Ok(_) => {}
                Err(e) => {
                    print_error!("{}", e);
                }
            },
            Err(e) => {
                print_error!("{}", e);
            }
        }
    }
    Ok(())
}
