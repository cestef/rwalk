use crate::_main;
use crate::cli::opts::Opts;
use anyhow::bail;
use anyhow::Result;
use colored::Colorize;
use lazy_static::lazy_static;
use log::error;
use rustyline::DefaultEditor;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

pub async fn main(mut opts: Opts) -> Result<()> {
    let mut rl = rustyline::DefaultEditor::new()?;

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
                    "help" | "h" | "?" => help(&mut rl, args, &mut opts).await,
                    "exit" | "quit" | "q" => exit(&mut rl, args, &mut opts).await,
                    "clear" | "cls" => clear(&mut rl, args, &mut opts).await,
                    "set" | "s" => set(&mut rl, args, &mut opts).await,
                    "append" | "a" => append(&mut rl, args, &mut opts).await,
                    "get" | "g" => get(&mut rl, args, &mut opts).await,
                    "list" | "ls" | "l" => list(&mut rl, args, &mut opts).await,
                    "run" | "r" => run(&mut rl, args, &mut opts).await,
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
    ];
}

async fn help(_rl: &mut DefaultEditor, _args: Vec<&str>, _state: &mut Opts) -> Result<()> {
    println!("Available commands:");
    for cmd in COMMANDS.iter() {
        println!("  {:<10} {}", cmd.name.bold(), cmd.description.dimmed());
    }
    Ok(())
}

async fn exit(_rl: &mut DefaultEditor, _args: Vec<&str>, _state: &mut Opts) -> Result<()> {
    std::process::exit(0);
}

async fn clear(rl: &mut DefaultEditor, _args: Vec<&str>, _state: &mut Opts) -> Result<()> {
    rl.clear_screen()?;
    Ok(())
}

fn get_field_by_name<T, R>(data: &T, field: &str) -> Result<R>
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

fn set_field_by_name<T>(data: &T, field: &str, value: &str) -> Result<T>
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

async fn set(_rl: &mut DefaultEditor, args: Vec<&str>, state: &mut Opts) -> Result<()> {
    if args.len() != 2 {
        println!("Usage: set <key> <value>");
        return Ok(());
    }
    let key = args[0];
    let value = args[1];

    let maybe_new_state = set_field_by_name(state, key, value);
    match maybe_new_state {
        Ok(new_state) => {
            *state = new_state;
            Ok(())
        }
        Err(e) => {
            error!("Error setting value: {}", e);
            Ok(())
        }
    }
}

async fn append(_rl: &mut DefaultEditor, args: Vec<&str>, state: &mut Opts) -> Result<()> {
    if args.len() != 2 {
        println!("Usage: append <key> <value>");
        return Ok(());
    }
    let key = args[0];
    let value = args[1];

    let current_value = get_field_by_name::<Opts, Value>(&state, key)?;
    if let Value::Array(mut vec) = current_value {
        vec.push(serde_json::from_str(value)?);
        let new_state = set_field_by_name(state, key, &serde_json::to_string(&vec)?)?;
        *state = new_state;
        println!("{} = {}", key, serde_json::to_string_pretty(&vec)?);
    } else {
        println!("{} is not an array", key);
    }
    Ok(())
}

async fn get(_rl: &mut DefaultEditor, args: Vec<&str>, state: &mut Opts) -> Result<()> {
    if args.len() != 1 {
        println!("Usage: get <key>");
        return Ok(());
    }
    let key = args[0];
    let maybe_value = get_field_by_name::<Opts, Value>(&state, key);
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

fn list_fields<T>(data: &T) -> Vec<(String, String)>
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

async fn list(_rl: &mut DefaultEditor, _args: Vec<&str>, state: &mut Opts) -> Result<()> {
    let fields = list_fields(state);
    let max_key_len = fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    for (key, value) in fields {
        println!(
            "{:<width$} = {}",
            key.bold(),
            value.dimmed(),
            width = max_key_len
        );
    }
    Ok(())
}

async fn run(_rl: &mut DefaultEditor, _args: Vec<&str>, state: &mut Opts) -> Result<()> {
    let res = _main(state.clone()).await;
    match res {
        Ok(_) => {}
        Err(e) => {
            error!("{}", e);
            return Ok(());
        }
    }
    Ok(())
}
