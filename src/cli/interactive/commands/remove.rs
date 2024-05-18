use crate::cli::{
    interactive::{get_field_by_name, set_field_by_name, State},
    opts::Opts,
};
use color_eyre::eyre::Result;
use log::error;
use serde_json::Value;
pub fn remove(args: Vec<&str>, state: &mut State) -> Result<()> {
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
