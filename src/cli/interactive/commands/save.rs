use std::{path::PathBuf, sync::Arc};

use crate::{
    cli::interactive::{Command, State},
    utils::constants::DEFAULT_CONFIG_PATH,
};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use colored::Colorize;
use rhai::{Engine, Scope};
use rustyline::DefaultEditor;
use tokio::sync::Mutex;
#[derive(Debug)]
pub struct SaveCommand;

#[async_trait]
impl Command for SaveCommand {
    fn name(&self) -> &'static str {
        "save"
    }

    fn description(&self) -> &'static str {
        "Save the current configuration to a file"
    }

    fn aliases(&self) -> Vec<&'static str> {
        vec!["s", "write"]
    }

    async fn run(
        &self,
        rl: Arc<Mutex<DefaultEditor>>,
        args: Vec<&str>,
        state: Arc<Mutex<State>>,
        _engine: Arc<Mutex<Engine>>,
        _scope: Arc<Mutex<Scope<'_>>>,
    ) -> Result<()> {
        let state = state.lock().await;
        let output = if args.len() == 1 {
            PathBuf::from(args[0])
        } else if let Some(home) = dirs::home_dir() {
            home.join(DEFAULT_CONFIG_PATH)
        } else {
            println!("Could not determine home directory");
            return Ok(());
        };
        let content = toml::to_string_pretty(&state.opts)?;
        // If the file already exists, prompt the user to confirm overwriting it
        if output.exists() {
            let mut rl = rl.lock().await;
            let response = rl.readline(&format!(
                "File {} already exists. Overwrite? [y/N]: ",
                output.to_string_lossy().bold()
            ))?;
            const YES: [&str; 2] = ["y", "yes"];
            if !YES.contains(&response.trim().to_lowercase().as_str()) {
                println!("Aborted");
                return Ok(());
            }
        }
        tokio::fs::write(&output, content).await?;
        println!("Configuration saved to {}", output.to_string_lossy().bold());
        Ok(())
    }
}
