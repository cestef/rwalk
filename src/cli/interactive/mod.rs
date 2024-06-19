use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use colored::Colorize;
use commands::{
    append::AppendCommand,
    eval::EvalCommand,
    get::GetCommand,
    list::ListCommand,
    load::LoadCommand,
    misc::{ClearCommand, ExitCommand},
    remove::RemoveCommand,
    run::RunCommand,
    save::SaveCommand,
    set::SetCommand,
};

use rhai::{exported_module, Engine, Scope};
use rustyline::DefaultEditor;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::{
    runner::scripting::ScriptingResponse,
    utils::tree::{tree_data, tree_node, TreeData, TreeNode},
};

use super::opts::Opts;
use color_eyre::eyre::{bail, Result};

mod commands;

#[derive(Debug)]
pub struct State {
    pub opts: Opts,
    pub last_result: Option<TreeNode<TreeData>>,
}

unsafe impl Send for State {}

#[async_trait]
pub trait Command: Debug {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn aliases(&self) -> Vec<&'static str> {
        vec![]
    }
    async fn run(
        &self,
        rl: Arc<Mutex<DefaultEditor>>,
        args: Vec<&str>,
        state: Arc<Mutex<State>>,
        engine: Arc<Mutex<Engine>>,
        scope: Arc<Mutex<Scope<'_>>>,
    ) -> Result<()>;
}

pub async fn main_interactive(opts: Opts) -> Result<()> {
    let rl = rustyline::DefaultEditor::new()?;
    let rl = Arc::new(Mutex::new(rl));
    let mut engine = Engine::new();

    let tree_module = exported_module!(tree_node);
    let tree_data_module = exported_module!(tree_data);
    engine.register_global_module(tree_module.into());
    engine.register_global_module(tree_data_module.into());

    engine.build_type::<TreeData>();
    engine.build_type::<ScriptingResponse>();
    let engine = Arc::new(Mutex::new(engine));
    let scope = Scope::new();
    let scope = Arc::new(Mutex::new(scope));
    let commands: Vec<Box<dyn Command>> = vec![
        Box::new(ExitCommand),
        Box::new(ClearCommand),
        Box::new(GetCommand),
        Box::new(SetCommand),
        Box::new(AppendCommand),
        Box::new(RemoveCommand),
        Box::new(EvalCommand),
        Box::new(RunCommand),
        Box::new(ListCommand),
        Box::new(SaveCommand),
        Box::new(LoadCommand),
    ];
    let state = State {
        opts,
        last_result: None,
    };
    let state = Arc::new(Mutex::new(state));

    loop {
        let mut rll = rl.lock().await;
        let readline = rll.readline(">> ");

        match readline {
            Ok(line) => {
                let line = line.clone();
                rll.add_history_entry(line.as_str())?;
                let parts = line.split(' ').collect::<Vec<_>>();
                let cmd = parts[0];
                if cmd.is_empty() {
                    continue;
                }
                let args = parts[1..].to_vec();
                if cmd == "help" || cmd == "?" {
                    if args.is_empty() {
                        println!("Available commands:");
                        for cmd in commands.iter() {
                            println!("  {:<10} {}", cmd.name().bold(), cmd.description().dimmed());
                        }
                    } else {
                        let cmd = commands.iter().find(|c| c.name() == args[0]);
                        match cmd {
                            Some(cmd) => {
                                println!("{}: {}", cmd.name().bold(), cmd.description());
                            }
                            None => {
                                println!("Command not found: {}", args[0]);
                            }
                        }
                    }
                } else {
                    let command = commands
                        .iter()
                        .find(|c| c.name() == cmd || c.aliases().contains(&cmd));
                    match command {
                        Some(command) => {
                            // Free rl lock before running the command
                            drop(rll);
                            command
                                .run(
                                    rl.clone(),
                                    args,
                                    state.clone(),
                                    engine.clone(),
                                    scope.clone(),
                                )
                                .await?;
                        }
                        None => {
                            println!("Unknown command: {}", cmd);
                        }
                    }
                }
            }
            Err(_) => break,
        }
    }
    Ok(())
}

pub fn get_field_by_name<T, R>(data: &T, field: &str) -> Result<R>
where
    T: Serialize,
    R: DeserializeOwned,
{
    let mut map = match serde_json::to_value(data) {
        Ok(Value::Object(map)) => map,
        _ => bail!("expected a struct"),
    };

    let value = match map.remove(field) {
        // remove the value from the map to get it without a reference
        Some(value) => value,
        None => bail!("field not found"),
    };

    R::deserialize(value).map_err(|e| e.into())
}

pub fn set_field_by_name<T>(data: &T, field: &str, value: &str) -> Result<T>
where
    T: Serialize + DeserializeOwned,
{
    let mut map = match serde_json::to_value(data) {
        Ok(Value::Object(map)) => map,
        _ => bail!("expected a struct"),
    };

    let value = serde_json::from_str(value)?;

    map.insert(field.to_string(), value);

    T::deserialize(Value::Object(map)).map_err(|e| e.into())
}

pub fn list_fields<T>(data: &T) -> Vec<(String, String)>
where
    T: Serialize,
{
    let map = match serde_json::to_value(data) {
        Ok(Value::Object(map)) => map,
        _ => return vec![],
    };

    map.iter()
        .map(|(k, v)| (k.clone(), v.to_string()))
        .collect()
}
