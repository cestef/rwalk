use std::sync::Arc;

use crate::cli::interactive::{set_field_by_name, Command, State};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use log::error;
use rhai::{Engine, Scope};
use rustyline::DefaultEditor;
use tokio::sync::Mutex;
#[derive(Debug)]
pub struct SetCommand;

#[async_trait]
impl Command for SetCommand {
    fn name(&self) -> &'static str {
        "set"
    }

    fn description(&self) -> &'static str {
        "Sets a value"
    }

    async fn run(
        &self,
        _rl: Arc<Mutex<DefaultEditor>>,
        args: Vec<&str>,
        state: Arc<Mutex<State>>,
        _engine: Arc<Mutex<Engine>>,
        _scope: Arc<Mutex<Scope<'_>>>,
    ) -> Result<()> {
        if args.len() != 2 {
            println!("Usage: set <key> <value>");
            return Ok(());
        }
        let key = args[0];
        let value = args[1];
        let mut state = state.lock().await;
        let maybe_new_state = set_field_by_name(&state.opts, key, value);
        match maybe_new_state {
            Ok(new_state) => {
                state.opts = new_state;
                Ok(())
            }
            Err(e) => {
                error!("Error setting value: {}", e);
                Ok(())
            }
        }
    }
}
