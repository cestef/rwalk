use std::sync::Arc;

use crate::cli::{
    interactive::{get_field_by_name, Command, State},
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
pub struct GetCommand;

#[async_trait]
impl Command for GetCommand {
    fn name(&self) -> &'static str {
        "get"
    }

    fn description(&self) -> &'static str {
        "Gets a value"
    }

    async fn run(
        &self,
        _rl: Arc<Mutex<DefaultEditor>>,
        args: Vec<&str>,
        state: Arc<Mutex<State>>,
        _engine: Arc<Mutex<Engine>>,
        _scope: Arc<Mutex<Scope<'_>>>,
    ) -> Result<()> {
        if args.len() != 1 {
            println!("Usage: get <key>");
            return Ok(());
        }
        let key = args[0];
        let state = state.lock().await;
        let maybe_value = get_field_by_name::<Opts, Value>(&state.opts, key);
        match maybe_value {
            Ok(value) => {
                println!("{}", serde_json::to_string_pretty(&value)?);
                Ok(())
            }
            Err(e) => {
                error!("Error getting value: {}", e);
                Ok(())
            }
        }
    }
}
