#![allow(dead_code)]

use std::{path::Path, process};

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

    let mut opts = Opts::parse();
    if let Some(p) = opts.config {
        opts = Opts::from_path(p.clone()).await?;
        log::info!("Using config file: {}", p);
    } else if let Some(home) = dirs::home_dir() {
        let p = home.join(Path::new(".config/rwalk/config.toml"));
        if p.exists() {
            opts = Opts::from_path(p.clone()).await?;
            log::info!("Using config file: {}", p.display());
        }
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
