#![allow(dead_code)]

use anyhow::Result;
use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::{generate, Generator, Shell};
use clap_complete_nushell::Nushell;
use log::error;
use merge::Merge;
use rwalk::{
    _main,
    cli::{self, opts::Opts},
    utils::{
        self,
        constants::{COMPLETIONS_PATH, DEFAULT_CONFIG_PATH},
    },
};
use std::{fs::OpenOptions, path::Path, process};

#[tokio::main]
async fn main() -> Result<()> {
    utils::logger::init_logger();

    let mut opts = Opts::parse();

    if let Some(p) = opts.config {
        opts = Opts::from_path(p.clone()).await?;
        log::debug!("Using config file: {}", p);
    } else if let Some(home) = dirs::home_dir() {
        log::debug!("Home directory found: {}", home.display());
        let p = home.join(Path::new(DEFAULT_CONFIG_PATH));
        if p.exists() {
            log::debug!("Config file found: {}", p.display());
            let path_opts = Opts::from_path(p.clone()).await?;
            opts.merge(path_opts);
            log::debug!("Using config file: {}", p.display());
        }
    } else {
        log::debug!("No home directory found");
    }

    log::debug!("Parsed options: {:#?}", opts);

    if opts.generate_markdown {
        clap_markdown::print_help_markdown::<Opts>();
        process::exit(0);
    }

    if opts.generate_completions {
        let name = env!("CARGO_PKG_NAME");
        let dir = Path::new(COMPLETIONS_PATH);
        if !dir.exists() {
            log::debug!("Creating completions directory: {}", dir.display());
            std::fs::create_dir_all(dir)?;
        }
        for s in Shell::value_variants().iter() {
            log::debug!("Generating completions for {}", s);
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(dir.join(s.file_name(name)))?;
            generate(*s, &mut Opts::command(), name, &mut file);
        }

        log::debug!("Generating completions for nushell");
        // Generate completions for nushell
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(dir.join(Nushell.file_name(name)))?;
        generate(Nushell, &mut Opts::command(), name, &mut file);

        log::info!("Generated completions");
        process::exit(0);
    }

    if opts.no_color {
        colored::control::set_override(false);
    }

    if !opts.quiet {
        utils::banner();
    }

    let res = if opts.interactive {
        cli::interactive::main(opts).await
    } else {
        _main(opts).await
    };
    if let Err(e) = res {
        error!("{}", e);
        process::exit(1);
    }
    process::exit(0);
}
