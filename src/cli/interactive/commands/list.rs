use std::sync::Arc;

use crate::cli::interactive::{list_fields, Command, State};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use colored::Colorize;
use rhai::{Engine, Scope};
use rustyline::DefaultEditor;
use tokio::sync::Mutex;
#[derive(Debug)]
pub struct ListCommand;

#[async_trait]
impl Command for ListCommand {
    fn name(&self) -> &'static str {
        "list"
    }

    fn description(&self) -> &'static str {
        "Lists all fields"
    }

    fn aliases(&self) -> Vec<&'static str> {
        vec!["ls", "l"]
    }

    async fn run(
        &self,
        _rl: Arc<Mutex<DefaultEditor>>,
        _args: Vec<&str>,
        state: Arc<Mutex<State>>,
        _engine: Arc<Mutex<Engine>>,
        _scope: Arc<Mutex<Scope<'_>>>,
    ) -> Result<()> {
        let state = state.lock().await;
        let fields = list_fields(&state.opts);
        let max_key_len = fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
        for (key, value) in fields {
            println!(
                "{} {dots} = {}",
                key.bold(),
                value.dimmed(),
                dots = "·".repeat(max_key_len - key.len()).dimmed(),
            );
        }
        Ok(())
    }
}
