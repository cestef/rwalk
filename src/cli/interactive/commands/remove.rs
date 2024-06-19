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
pub struct RemoveCommand;

#[async_trait]
impl Command for RemoveCommand {
    fn name(&self) -> &'static str {
        "remove"
    }

    fn description(&self) -> &'static str {
        "Removes a value from an array"
    }

    fn aliases(&self) -> Vec<&'static str> {
        vec!["delete", "rm"]
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
            println!("Usage: remove <key> <value|index>");
            return Ok(());
        }
        let key = args[0];
        let value = args[1];
        let mut state = state.lock().await;
        let current_value = get_field_by_name::<Opts, Value>(&state.opts, key)?;
        if let Value::Array(vec) = current_value {
            if let Ok(index) = value.parse::<usize>() {
                if index >= vec.len() {
                    println!("Index out of bounds");
                    return Ok(());
                }
                let new_vec = vec
                    .into_iter()
                    .enumerate()
                    .filter(|(i, _)| i != &index)
                    .map(|(_, v)| v)
                    .collect::<Vec<Value>>();
                let new_value = Value::Array(new_vec);
                let maybe_new_state =
                    set_field_by_name(&state.opts, key, &serde_json::to_string(&new_value)?);
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
            } else {
                let new_vec = vec
                    .into_iter()
                    .filter(|v| v != value)
                    .collect::<Vec<Value>>();
                let new_value = Value::Array(new_vec);
                let maybe_new_state =
                    set_field_by_name(&state.opts, key, &serde_json::to_string(&new_value)?);
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
        } else {
            println!("Value is not an array");
            Ok(())
        }
    }
}
