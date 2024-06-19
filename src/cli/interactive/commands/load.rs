use std::{path::PathBuf, sync::Arc};

use crate::{
    cli::interactive::{Command, State},
    utils::constants::DEFAULT_CONFIG_PATH,
};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use colored::Colorize;
use merge::Merge;
use rhai::{Engine, Scope};
use rustyline::DefaultEditor;
use tokio::sync::Mutex;
#[derive(Debug)]
pub struct LoadCommand;

#[async_trait]
impl Command for LoadCommand {
    fn name(&self) -> &'static str {
        "load"
    }

    fn description(&self) -> &'static str {
        "Load a configuration file"
    }

    fn aliases(&self) -> Vec<&'static str> {
        vec!["read"]
    }

    async fn run(
        &self,
        _rl: Arc<Mutex<DefaultEditor>>,
        args: Vec<&str>,
        state: Arc<Mutex<State>>,
        _engine: Arc<Mutex<Engine>>,
        _scope: Arc<Mutex<Scope<'_>>>,
    ) -> Result<()> {
        let mut state = state.lock().await;
        let input = if args.len() == 1 {
            PathBuf::from(args[0])
        } else if let Some(home) = dirs::home_dir() {
            home.join(DEFAULT_CONFIG_PATH)
        } else {
            println!("Could not determine home directory");
            return Ok(());
        };
        let content = std::fs::read_to_string(&input)?;
        let opts: crate::cli::opts::Opts = toml::from_str(&content)?;
        let merge = args.iter().any(|&arg| arg == "--merge" || arg == "-m");
        if merge {
            state.opts.merge(opts);
        } else {
            state.opts = opts;
        }
        println!(
            "Loaded configuration from {}",
            input.to_string_lossy().bold()
        );
        Ok(())
    }
}
