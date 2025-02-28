use std::io::BufWriter;

use clap::{ArgAction, CommandFactory as _, Parser};

use merge::Merge;
use rwalk::{
    cli::{utils, Opts},
    run, RwalkError,
};
use tabled::settings::{
    object::{Columns, Object, Rows},
    style::BorderColor,
    Alignment, Color, Style,
};
use termimad::{ansi, MadSkin};
use tracing::debug;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> miette::Result<()> {
    let indicatif_layer = IndicatifLayer::new();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .with(
            EnvFilter::from_env("RWALK_LOG")
                .add_directive(
                    "hyper_util=off"
                        .parse()
                        .map_err(|e| miette::miette!("Failed to parse directive: {}", e))?,
                )
                .add_directive(
                    "reqwest=off"
                        .parse()
                        .map_err(|e| miette::miette!("Failed to parse directive: {}", e))?,
                ),
        )
        .init();

    let mut opts = Opts::parse();
    debug!("{:#?}", opts);
    if let Some(ref config) = opts.config {
        let config = tokio::fs::read_to_string(config)
            .await
            .map_err(RwalkError::from)?;
        let config: Opts = toml::from_str(&config).map_err(RwalkError::from)?;
        opts.merge(config);
        debug!("merged: {:#?}", opts);
    }

    // println!("{}", table::from_opts(&opts));

    if opts.help {
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

        let mut table = tabled::builder::Builder::new();

        table.push_record(["short", "long", "description"]);

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
                help.push_str(&format!("* `{}`: {}\n", key, h));
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

**Usage:** `{name}{args}`
{help}
**Options:**
",
            version = env!("CARGO_PKG_VERSION"),
            name = cmd.get_name(),
        );
        skin.print_text(&intro);

        let options = cmd
            .get_arguments()
            .filter(|a| !a.is_hide_set())
            .filter(|a| a.get_short().is_some() || a.get_long().is_some());

        for option in options {
            let value = match option.get_action() {
                ArgAction::Append | ArgAction::Set => option
                    .get_value_names()
                    .map_or("".to_string(), |v| format!("<{}>", v[0])),
                _ => "".to_string(),
            };
            table.push_record([
                option
                    .get_short()
                    .map_or("".to_string(), |s| format!("-{s}")),
                {
                    let aliases = option.get_visible_aliases().unwrap_or_default();
                    let to_render = option.get_long().map_or("".to_string(), |s| {
                        format!(
                            "--{s}{aliases} *{value}*",
                            aliases = if !aliases.is_empty() {
                                format!(", --{}", aliases.join(", "))
                            } else {
                                "".to_string()
                            }
                        )
                    });
                    let mut out = BufWriter::new(Vec::new());
                    let _ = skin.write_inline_on(&mut out, &to_render);
                    String::from_utf8(out.buffer().to_vec())
                        .unwrap()
                        .trim_end_matches('\n')
                        .to_string()
                },
                option.get_help().map_or("".to_string(), |s| {
                    let mut help = format!("{}", s.ansi());

                    let mut possible_values = option.get_possible_values();
                    if !possible_values.is_empty() {
                        let possible_values: Vec<String> = possible_values
                            .drain(..)
                            .map(|v| format!("`{}`", v.get_name()))
                            .collect();
                        help.push_str(&format!(
                            "\n* Possible values: {}",
                            possible_values.join(", ")
                        ));
                    }
                    if let Some(default) = option.get_default_values().first() {
                        match option.get_action() {
                            ArgAction::Set | ArgAction::Append => {
                                help.push_str(&format!(
                                    "\n* Default: `{}`",
                                    default.to_string_lossy()
                                ));
                            }
                            _ => {}
                        }
                    }

                    let mut out = BufWriter::new(Vec::new());

                    let _ = skin.write_text_on(&mut out, &help);

                    String::from_utf8(out.buffer().to_vec())
                        .unwrap()
                        .trim_end_matches('\n')
                        .to_string()
                }),
            ]);
        }

        println!(
            "{}",
            table
                .build()
                .with(Style::modern_rounded())
                .modify(
                    Rows::new(0..=1).and(Columns::new(0..=2)),
                    BorderColor::filled(Color::new("\u{1b}[2m", "\u{1b}[0m"))
                )
                .modify(Columns::first(), Alignment::center())
        );

        return Ok(());
    }

    if opts.list_filters {
        utils::list_filters();
        return Ok(());
    }

    if opts.list_transforms {
        utils::list_transforms();
        return Ok(());
    }

    run(opts).await?;

    Ok(())
}
