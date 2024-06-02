use std::sync::Arc;

use crate::cli::interactive::{Command, State};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use rhai::{Engine, Scope};
use rustyline::DefaultEditor;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct ExitCommand;

#[async_trait]
impl Command for ExitCommand {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn description(&self) -> &'static str {
        "Exits the interactive shell"
    }

    async fn run(
        &self,
        _rl: Arc<Mutex<DefaultEditor>>,
        _args: Vec<&str>,
        _state: Arc<Mutex<State>>,
        _engine: Arc<Mutex<Engine>>,
        _scope: Arc<Mutex<Scope<'_>>>,
    ) -> Result<()> {
        std::process::exit(0);
    }
}
#[derive(Debug)]
pub struct ClearCommand;

#[async_trait]
impl Command for ClearCommand {
    fn name(&self) -> &'static str {
        "clear"
    }

    fn description(&self) -> &'static str {
        "Clears the screen"
    }

    async fn run(
        &self,
        rl: Arc<Mutex<DefaultEditor>>,
        _args: Vec<&str>,
        _state: Arc<Mutex<State>>,
        _engine: Arc<Mutex<Engine>>,
        _scope: Arc<Mutex<Scope<'_>>>,
    ) -> Result<()> {
        let mut rl = rl.lock().await;
        rl.clear_screen()?;
        Ok(())
    }
}
