use crate::cli::{
    interactive::{get_field_by_name, set_field_by_name, State},
    opts::Opts,
};
use color_eyre::eyre::Result;
use log::error;
use serde_json::Value;

pub fn append(args: Vec<&str>, state: &mut State) -> Result<()> {
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
