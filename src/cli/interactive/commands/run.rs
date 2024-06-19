use std::sync::Arc;

use crate::{
    _main,
    cli::interactive::{Command, State},
};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use log::error;
use rhai::{Engine, Scope};
use rustyline::DefaultEditor;
use tokio::sync::Mutex;
#[derive(Debug)]
pub struct RunCommand;

#[async_trait]
impl Command for RunCommand {
    fn name(&self) -> &'static str {
        "run"
    }

    fn description(&self) -> &'static str {
        "Runs the current tree"
    }

    fn aliases(&self) -> Vec<&'static str> {
        vec!["r", "scan", "exec", "go"]
    }

    async fn run(
        &self,
        _rl: Arc<Mutex<DefaultEditor>>,
        _args: Vec<&str>,
        state: Arc<Mutex<State>>,
        _engine: Arc<Mutex<Engine>>,
        _scope: Arc<Mutex<Scope<'_>>>,
    ) -> Result<()> {
        let mut state = state.lock().await;
        let res = _main(state.opts.clone()).await;
        match res {
            Ok(r) => {
                if let Some(root) = r.root {
                    state.last_result = Some(root.lock().clone());
                }
            }
            Err(e) => {
                error!("{}", e);
                return Ok(());
            }
        }
        Ok(())
    }
}
