#![allow(dead_code)]

use std::process;

use anyhow::Result;
use clap::Parser;
use log::error;
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
    let mut opts = Opts::parse();
    if let Some(p) = opts.config {
        opts = Opts::from_path(p).await?;
    }

    if opts.generate_markdown {
        clap_markdown::print_help_markdown::<Opts>();
        process::exit(0);
    }
    if opts.no_color {
        colored::control::set_override(false);
    }
    if !opts.quiet {
        utils::banner();
    }
    let res = if opts.interactive {
        cli::interactive::main().await
    } else {
        _main(opts.clone()).await
    };
    if let Err(e) = res {
        error!("{}", e);
        process::exit(1);
    }
    process::exit(0);
}
