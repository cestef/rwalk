use lazy_static::lazy_static;
use rhai::{exported_module, Engine, Scope};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::utils::tree::{tree, TreeData, TreeNode};

use self::commands::{
    append::append,
    eval::eval,
    get::get,
    list::list,
    misc::{clear, exit, help},
    remove::remove,
    run::run,
    set::set,
};

use super::opts::Opts;
use color_eyre::eyre::{bail, Result};

mod commands;

lazy_static! {
    static ref COMMANDS: Vec<Command> = vec![
        Command::new("help", "Show this help message"),
        Command::new("exit", "Exit the program"),
        Command::new("clear", "Clear the screen"),
        Command::new("set", "Set a value"),
        Command::new("unset", "Unset a value"),
        Command::new("get", "Get a value"),
        Command::new("list", "List all values"),
        Command::new("run", "Run the scanner"),
        Command::new("append", "Append a value to an array"),
        Command::new("remove", "Remove a value from an array"),
    ];
}

pub struct State {
    opts: Opts,
    last_result: Option<TreeNode<TreeData>>,
}

pub async fn main_interactive(opts: Opts) -> Result<()> {
    let mut rl = rustyline::DefaultEditor::new()?;
    let mut engine = Engine::new();
    let tree_module = exported_module!(tree);
    engine.register_global_module(tree_module.into());
    let mut scope = Scope::new();

    let mut state = State {
        opts,
        last_result: None,
    };

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let parts = line.split(' ').collect::<Vec<_>>();
                let cmd = parts[0];
                if cmd.is_empty() {
                    continue;
                }
                let args = parts[1..].to_vec();
                // This is a bit ugly, but I can't manage to box async functions
                match cmd {
                    "help" | "h" | "?" => help(),
                    "exit" | "quit" | "q" => exit(),
                    "clear" | "cls" => clear(&mut rl),
                    "set" | "s" => set(args, &mut state),
                    "append" | "a" => append(args, &mut state),
                    "get" | "g" => get(args, &mut state),
                    "list" | "ls" | "l" => list(&mut state),
                    "run" | "r" => run(&mut state).await,
                    "remove" | "rm" => remove(args, &mut state),
                    "eval" | "e" => eval(&mut rl, args, &mut state, &mut engine, &mut scope),
                    _ => {
                        println!("Unknown command: {}", cmd);
                        Ok(())
                    }
                }?;
            }
            Err(_) => break,
        }
    }
    Ok(())
}

struct Command {
    name: String,
    description: String,
}

impl Command {
    fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
        }
    }
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
