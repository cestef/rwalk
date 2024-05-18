use crate::cli::{
    interactive::{get_field_by_name, State},
    opts::Opts,
};
use color_eyre::eyre::Result;
use log::error;
use serde_json::Value;

pub fn get(args: Vec<&str>, state: &mut State) -> Result<()> {
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
