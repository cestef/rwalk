use clap::CommandFactory;
use termimad::{ansi, MadSkin};

use super::Opts;

pub fn print(long: bool) {
    let mut skin = match terminal_light::luma() {
        Ok(luma) if luma > 0.85 => MadSkin::default_light(),
        Ok(luma) if luma < 0.2 => MadSkin::default_dark(),
        _ => MadSkin::default(),
    };

    skin.headers[0].compound_style.set_fg(ansi(210));
    skin.bold.set_fg(ansi(210));
    skin.italic = termimad::CompoundStyle::with_fg(ansi(210));
    skin.strikeout =
        termimad::CompoundStyle::with_attr(termimad::crossterm::style::Attribute::Underdashed);

    let mut cmd = Opts::command();
    cmd.build();

    let mut args = String::new();
    let mut help = String::new();
    for arg in cmd.get_positionals() {
        let Some(key) = arg.get_value_names().and_then(|arr| arr.first()) else {
            continue;
        };
        args.push(' ');
        if !arg.is_required_set() {
            args.push('[');
        }
        if arg.is_last_set() {
            args.push_str("-- ");
        }
        args.push_str(key);
        if !arg.is_required_set() {
            args.push(']');
        }
        if let Some(h) = arg.get_help() {
            help.push_str(&format!("\n* `{}`: {}", key, h));
        }
    }

    let intro = format!(
        "
## **rwalk** v{version}

A blazingly fast web fuzzer with granular filtering and transformation capabilities.

It supports:
- fuzzing modes: *r* for `recursive` and *t* for `template`
- filters: *status*, *headers*, ... (see `--list-filters`)
- transforms: *encode*, *case*, ... (see `--list-transforms`)

Complete documentation is available at ~~https://rwalk.cstef.dev~~

**Usage:** `{name}{args}`{help}

{}
",
        if long {
            "**Options:**"
        } else {
            "Use `--help` for a list of options."
        },
        version = env!("CARGO_PKG_VERSION"),
        name = cmd.get_name(),
    );
    skin.print_text(&intro);
    if !long {
        return;
    }
    let _ = Opts::command().print_long_help();
}
