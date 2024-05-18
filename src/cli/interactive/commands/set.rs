use crate::cli::interactive::{set_field_by_name, State};
use color_eyre::eyre::Result;
use log::error;

pub fn set(args: Vec<&str>, state: &mut State) -> Result<()> {
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
