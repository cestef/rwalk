use crate::cli::interactive::{list_fields, State};
use color_eyre::eyre::Result;
use colored::Colorize;

pub fn list(state: &mut State) -> Result<()> {
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
