#![allow(dead_code)]

use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::{Generator, Shell};
use clap_complete_nushell::Nushell;
use color_eyre::eyre::{eyre, Result};
use log::error;
use merge::Merge;
use rwalk::{
    _main,
    cli::{self, opts::Opts},
    utils::{self, constants::DEFAULT_CONFIG_PATH},
};
use std::{path::Path, process};

#[tokio::main]
async fn main() -> Result<()> {
    utils::logger::init_logger();
    utils::init_panic()?;
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

    if let Some(ref shell) = opts.completions {
        // Generate completions for the specified shell and print them to the console
        let name = env!("CARGO_PKG_NAME");
        let mut stream = std::io::stdout();
        let shell: Box<dyn Generator> = match shell.to_lowercase().as_str() {
            "nushell" => Box::new(Nushell),
            _ => Box::new(
                Shell::from_str(shell, true)
                    .map_err(|e| eyre!("Invalid shell: {}. Error: {}", shell, e))?,
            ),
        };
        let mut cmd = Opts::command();
        cmd.set_bin_name(name);
        cmd.build();
        shell.generate(&cmd, &mut stream);
        process::exit(0);
    }

    if opts.no_color {
        colored::control::set_override(false);
    }

    let res = if opts.interactive {
        cli::interactive::main_interactive(opts).await
    } else {
        _main(opts).await.map(|_| ())
    };
    if let Err(e) = res {
        error!("{}", e);
        process::exit(1);
    }
    process::exit(0);
}
