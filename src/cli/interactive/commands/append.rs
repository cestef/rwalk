use std::sync::Arc;

use crate::cli::{
    interactive::{get_field_by_name, set_field_by_name, Command, State},
    opts::Opts,
};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use log::error;
use rhai::{Engine, Scope};
use rustyline::DefaultEditor;
use serde_json::Value;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct AppendCommand;

#[async_trait]
impl Command for AppendCommand {
    fn name(&self) -> &'static str {
        "append"
    }

    fn description(&self) -> &'static str {
        "Appends a value to an array"
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
            println!("Usage: append <key> <value>");
            return Ok(());
        }
        let key = args[0];
        let value = args[1];
        let mut state = state.lock().await;
        let maybe_current_value = get_field_by_name::<Opts, Value>(&state.opts, key);
        let current_value = match maybe_current_value {
            Ok(value) => value,
            Err(e) => {
                error!("Error getting value: {}", e);
                return Ok(());
            }
        };
        if let Value::Array(mut vec) = current_value {
            vec.push(serde_json::from_str(value)?);
            let maybe_new_state =
                set_field_by_name(&state.opts, key, &serde_json::to_string(&vec)?);
            match maybe_new_state {
                Ok(new_state) => {
                    state.opts = new_state;
                    println!("{} = {}", key, serde_json::to_string_pretty(&vec)?);
                }
                Err(e) => {
                    error!("Error setting value: {}", e);
                }
            }
        } else {
            println!("{} is not an array", key);
        }
        Ok(())
    }
}
