use crate::_main;
use crate::cli::opts::Opts;
use crate::utils::tree::tree;
use crate::utils::tree::TreeData;
use crate::utils::tree::TreeNode;
use anyhow::bail;
use anyhow::Result;
use colored::Colorize;
use lazy_static::lazy_static;
use log::error;
use rhai::exported_module;
use rhai::Dynamic;
use rhai::Engine;
use rhai::Scope;
use rustyline::DefaultEditor;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

pub struct State {
    opts: Opts,
    last_result: Option<TreeNode<TreeData>>,
}

pub async fn main(opts: Opts) -> Result<()> {
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
                    "help" | "h" | "?" => help(&mut rl, args, &mut state).await,
                    "exit" | "quit" | "q" => exit(&mut rl, args, &mut state).await,
                    "clear" | "cls" => clear(&mut rl, args, &mut state).await,
                    "set" | "s" => set(&mut rl, args, &mut state).await,
                    "append" | "a" => append(&mut rl, args, &mut state).await,
                    "get" | "g" => get(&mut rl, args, &mut state).await,
                    "list" | "ls" | "l" => list(&mut rl, args, &mut state).await,
                    "run" | "r" => run(&mut rl, args, &mut state).await,
                    "remove" | "rm" => remove(&mut rl, args, &mut state).await,
                    "eval" | "e" => eval(&mut rl, args, &mut state, &mut engine, &mut scope).await,
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
        Command::new("append", "Append a value to an array"),
        Command::new("remove", "Remove a value from an array"),
    ];
}

async fn help(_rl: &mut DefaultEditor, _args: Vec<&str>, _state: &mut State) -> Result<()> {
    println!("Available commands:");
    for cmd in COMMANDS.iter() {
        println!("  {:<10} {}", cmd.name.bold(), cmd.description.dimmed());
    }
    Ok(())
}

async fn exit(_rl: &mut DefaultEditor, _args: Vec<&str>, _state: &mut State) -> Result<()> {
    std::process::exit(0);
}

async fn clear(rl: &mut DefaultEditor, _args: Vec<&str>, _state: &mut State) -> Result<()> {
    rl.clear_screen()?;
    Ok(())
}

async fn remove(_rl: &mut DefaultEditor, args: Vec<&str>, state: &mut State) -> Result<()> {
    if args.len() != 2 {
        println!("Usage: remove <key> <value>");
        return Ok(());
    }
    let key = args[0];
    let value = args[1];

    let current_value = get_field_by_name::<Opts, Value>(&state.opts, key)?;
    if let Value::Array(vec) = current_value {
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
    } else {
        println!("Value is not an array");
        Ok(())
    }
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

async fn set(_rl: &mut DefaultEditor, args: Vec<&str>, state: &mut State) -> Result<()> {
    if args.len() != 2 {
        println!("Usage: set <key> <value>");
        return Ok(());
    }
    let key = args[0];
    let value = args[1];

    let maybe_new_state = set_field_by_name(&state.opts, key, value);
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

async fn append(_rl: &mut DefaultEditor, args: Vec<&str>, state: &mut State) -> Result<()> {
    if args.len() != 2 {
        println!("Usage: append <key> <value>");
        return Ok(());
    }
    let key = args[0];
    let value = args[1];

    let maybe_current_value = get_field_by_name::<Opts, Value>(&state.opts, key);
    let current_value = match maybe_current_value {
        Ok(value) => value,
        Err(e) => {
            error!("Error getting value: {}", e);
            return Ok(());
        }
    };
    if let Value::Array(mut vec) = current_value {
        vec.push(serde_json::from_str(value)?);
        let maybe_new_state = set_field_by_name(&state.opts, key, &serde_json::to_string(&vec)?);
        match maybe_new_state {
            Ok(new_state) => {
                state.opts = new_state;
                println!("{} = {}", key, serde_json::to_string_pretty(&vec)?);
            }
            Err(e) => {
                error!("Error setting value: {}", e);
            }
        }
    } else {
        println!("{} is not an array", key);
    }
    Ok(())
}

async fn get(_rl: &mut DefaultEditor, args: Vec<&str>, state: &mut State) -> Result<()> {
    if args.len() != 1 {
        println!("Usage: get <key>");
        return Ok(());
    }
    let key = args[0];
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

async fn list(_rl: &mut DefaultEditor, _args: Vec<&str>, state: &mut State) -> Result<()> {
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

async fn run(_rl: &mut DefaultEditor, _args: Vec<&str>, state: &mut State) -> Result<()> {
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

async fn eval(
    _rl: &mut DefaultEditor,
    args: Vec<&str>,
    state: &mut State,
    engine: &mut Engine,
    scope: &mut Scope<'_>,
) -> Result<()> {
    if let Some(last_result) = &state.last_result {
        scope.set_or_push("tree", last_result.clone());
    }
    scope.set_or_push("opts", state.opts.clone());
    if args.is_empty() {
        // Enter interactive mode
        loop {
            let readline = _rl.readline("eval> ");
            match readline {
                Ok(mut line) => {
                    line = line.trim().to_string();
                    if line.is_empty() {
                        continue;
                    }
                    match line.as_str() {
                        "exit" | "quit" | "q" => break,
                        "clear" | "cls" => {
                            _rl.clear_screen()?;
                            continue;
                        }
                        _ => {}
                    }
                    _rl.add_history_entry(line.as_str())?;
                    execute(engine, scope, line)?;
                }
                Err(_) => break,
            }
        }
    } else {
        let line = args.join(" ");
        execute(engine, scope, line)?;
    }

    Ok(())
}

fn execute(engine: &mut Engine, scope: &mut Scope, line: String) -> Result<()> {
    let maybe_out = engine.eval_with_scope::<Dynamic>(scope, &line);
    match maybe_out {
        Ok(out) => {
            let out = out.to_string().trim().to_string();
            if out.is_empty() {
                return Ok(());
            }
            println!("{}", out);
        }
        Err(e) => {
            error!("{}", e);
        }
    }
    Ok(())
}
