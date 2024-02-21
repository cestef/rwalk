#![allow(dead_code)]

use std::process;

use anyhow::Result;
use clap::Parser;
use rwalk::{
    _main,
    cli::{self, opts::Opts},
    utils,
};

#[tokio::main]
async fn main() -> Result<()> {
    utils::logger::init_logger();
    let config_path = dirs::home_dir()
        .unwrap()
        .join(".config")
        .join("rwalk")
        .join(".env");
    dotenv::from_path(config_path).ok();
    let opts = Opts::parse();
    if opts.generate_markdown {
        clap_markdown::print_help_markdown::<Opts>();
        return Ok(());
    }
    if opts.no_color {
        colored::control::set_override(false);
    }
    if !opts.quiet {
        utils::banner();
    }
    if opts.interactive {
        cli::interactive::main().await?;
        process::exit(0);
    } else {
        _main(opts.clone()).await?;
        process::exit(0);
    }
}
