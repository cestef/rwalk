use crate::_main;
use crate::cli::opts::Opts;
use crate::cli::opts::OptsGetterSetter;
use crate::utils::parse_range_input;
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use lazy_static::lazy_static;
use rustyline::DefaultEditor;

pub async fn main() -> Result<()> {
    let mut rl = rustyline::DefaultEditor::new()?;
    let mut state = Opts::parse();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let parts = line.split(" ").collect::<Vec<_>>();
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
                    "unset" | "u" => unset(&mut rl, args, &mut state).await,
                    "get" | "g" => get(&mut rl, args, &mut state).await,
                    "list" | "ls" | "l" => list(&mut rl, args, &mut state).await,
                    "run" | "r" => run(&mut rl, args, &mut state).await,
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

async fn set(_rl: &mut DefaultEditor, args: Vec<&str>, state: &mut Opts) -> Result<()> {
    if args.len() != 2 {
        println!("Usage: set <key> <value>");
        return Ok(());
    }
    let key = args[0];
    let value = args[1];
    let value_type = get_value_type(value);

    let res = match value_type {
        ValueType::String => state.set(&key.to_string(), Some(value.to_string())),
        ValueType::Bool => state.set(&key.to_string(), parse_bool(value)),
        ValueType::Usize => state.set(&key.to_string(), Some(value.parse::<usize>().unwrap())),
        ValueType::Range => state.set(&key.to_string(), Some(value.to_string())),
        ValueType::StringVec => {
            let re = regex::Regex::new(r#"\[(.*)\]"#).unwrap();
            let value = re.replace_all(value, "$1").to_string();
            let value = value.split(",").map(|s| s.to_string()).collect::<Vec<_>>();
            state.set(&key.to_string(), value)
        }
    };

    match res {
        Ok(_) => {}
        Err(_) => {
            // Try to set the value as a string
            let res = state.set(&key.to_string(), Some(value.to_string()));
            match res {
                Ok(_) => {}
                Err(e) => {
                    println!("Error: {}", e);
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}

async fn append(_rl: &mut DefaultEditor, args: Vec<&str>, state: &mut Opts) -> Result<()> {
    if args.len() != 2 {
        println!("Usage: append <key> <value>");
        return Ok(());
    }
    let key = args[0];
    let value = args[1];

    let re = regex::Regex::new(r#"\[(.*)\]"#).unwrap();
    let current_value = match state.getenum(&key.to_string()) {
        Ok(value) => {
            let s = format!("{:?}", value);
            // depth(Some(1)) or depth(None)
            let re = regex::Regex::new(format!(r#"{}\(Some\((.*)\)\)"#, key).as_str()).unwrap();
            let s = re.replace_all(&s, "$1").to_string();
            // depth(1)
            let re = regex::Regex::new(format!(r#"{}\((.*)\)"#, key).as_str()).unwrap();
            let s = re.replace_all(&s, "$1").to_string();
            s
        }
        Err(_) => "".to_string(),
    };
    let current_value = re.replace_all(&current_value, "$1").to_string();
    let mut current_value = current_value
        .split(",")
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let mut value = value.split(",").map(|s| s.to_string()).collect::<Vec<_>>();
    current_value.append(&mut value);
    let res = state.set(
        &key.to_string(),
        current_value
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    match res {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
            return Ok(());
        }
    }

    Ok(())
}

#[derive(Debug)]
enum ValueType {
    String,
    Bool,
    Usize,
    Range,
    StringVec,
}

fn parse_bool(s: &str) -> bool {
    match s.to_lowercase().as_str() {
        "true" => true,
        "false" => false,
        _ => false,
    }
}

fn get_value_type(s: &str) -> ValueType {
    if s.starts_with("[") && s.ends_with("]") {
        return ValueType::StringVec;
    }
    if ["true", "false"].contains(&s.to_lowercase().as_str()) {
        return ValueType::Bool;
    }
    if s.parse::<usize>().is_ok() {
        return ValueType::Usize;
    }
    if parse_range_input(s).is_ok() {
        return ValueType::Range;
    }
    ValueType::String
}

async fn unset(_rl: &mut DefaultEditor, args: Vec<&str>, state: &mut Opts) -> Result<()> {
    if args.len() != 1 {
        println!("Usage: unset <key>");
        return Ok(());
    }
    let key = args[0];
    let res = state.set(&key.to_string(), None as Option<String>);
    match res {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
            return Ok(());
        }
    }

    Ok(())
}

async fn get(_rl: &mut DefaultEditor, args: Vec<&str>, state: &mut Opts) -> Result<()> {
    if args.len() != 1 {
        println!("Usage: get <key>");
        return Ok(());
    }
    let key = args[0];
    let value = match state.getenum(&key.to_string()) {
        Ok(value) => {
            let s = format!("{:?}", value);
            // depth(Some(1)) or depth(None)
            let re = regex::Regex::new(format!(r#"{}\(Some\((.*)\)\)"#, key).as_str()).unwrap();
            let s = re.replace_all(&s, "$1").to_string();
            // depth(1)
            let re = regex::Regex::new(format!(r#"{}\((.*)\)"#, key).as_str()).unwrap();
            let s = re.replace_all(&s, "$1").to_string();
            s
        }
        Err(_) => {
            println!("Unknown key: {}", key);
            return Ok(());
        }
    };

    println!("{}: {}", key, value);
    Ok(())
}

async fn list(_rl: &mut DefaultEditor, _args: Vec<&str>, state: &mut Opts) -> Result<()> {
    println!("Listing all values:");
    let struct_info = state.getstructinfo();
    let mut fields: Vec<(String, String, String)> =
        vec![("".to_string(), "".to_string(), "".to_string()); struct_info.field_names.len()];
    for (i, name) in struct_info.field_names.iter().enumerate() {
        fields[i].0 = name.to_string();
        let value = match state.getenum(&name.to_string()) {
            Ok(value) => {
                let s = format!("{:?}", value);
                // depth(Some(1)) or depth(None)
                let re =
                    regex::Regex::new(format!(r#"{}\(Some\((.*)\)\)"#, name).as_str()).unwrap();
                let s = re.replace_all(&s, "$1").to_string();
                // depth(1)
                let re = regex::Regex::new(format!(r#"{}\((.*)\)"#, name).as_str()).unwrap();
                let s = re.replace_all(&s, "$1").to_string();
                s
            }
            Err(_) => "".to_string(),
        };
        fields[i].2 = value;
    }
    for (i, ty) in struct_info.field_types.iter().enumerate() {
        let re = regex::Regex::new(r#"Option < (.*) >"#).unwrap();
        let ty = re.replace_all(ty, "$1").to_string();
        let re = regex::Regex::new(r#"Vec < (.*) >"#).unwrap();
        let ty = re.replace_all(&ty, "Vec<$1>").to_string();
        fields[i].1 = ty;
    }

    let max_len = fields
        .iter()
        .map(|(name, _, _)| name.len())
        .max()
        .unwrap_or(0);

    for (name, ty, value) in fields {
        println!(
            "  {:<width$} {} = {}",
            name.bold(),
            ty.dimmed(),
            value,
            width = max_len,
        );
    }
    Ok(())
}

async fn run(_rl: &mut DefaultEditor, _args: Vec<&str>, state: &mut Opts) -> Result<()> {
    let _ = _main(state.clone()).await;
    Ok(())
}
