use crate::cli::interactive::COMMANDS;
use color_eyre::eyre::Result;
use colored::Colorize;
use rustyline::DefaultEditor;

pub fn help() -> Result<()> {
    println!("Available commands:");
    for cmd in COMMANDS.iter() {
        println!("  {:<10} {}", cmd.name.bold(), cmd.description.dimmed());
    }
    Ok(())
}

pub fn exit() -> Result<()> {
    std::process::exit(0);
}

pub fn clear(rl: &mut DefaultEditor) -> Result<()> {
    rl.clear_screen()?;
    Ok(())
}
