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
use std::{
    path::{Path, PathBuf},
    process,
};

#[tokio::main]
async fn main() -> Result<()> {
    utils::logger::init_logger();
    utils::init_panic()?;

    // Step 1: Parse command-line arguments first (highest priority)
    let cli_opts = Opts::parse();

    // Step 2: Start with default values
    let mut opts = Opts::default();

    // Step 3: Load configuration file from CLI if provided, else fallback to default path in home directory
    let config_path = cli_opts
        .config
        .as_deref()
        .map(Path::new)
        .filter(|p| {
            if p.exists() {
                true
            } else {
                log::warn!(
                    "Config file not found: {}. Falling back to default.",
                    p.display()
                );
                false
            }
        })
        .map(Path::to_path_buf)
        .or_else(|| match dirs::home_dir() {
            Some(home) => {
                let default_path = home.join(DEFAULT_CONFIG_PATH);
                if default_path.exists() {
                    Some(default_path)
                } else {
                    log::debug!("Default config file not found: {}", default_path.display());
                    None
                }
            }
            None => {
                log::debug!("No home directory found; skipping default config file");
                None
            }
        });

    if let Some(config_path) = config_path {
        let config_opts = Opts::from_path(config_path.clone()).await?;
        opts.merge(config_opts);
        log::debug!("Loaded config file: {}", config_path.display());
    }

    // Step 4: Finally, override all options with CLI arguments
    opts.merge(cli_opts);
    log::debug!("Final merged options: {:#?}", opts);

    if opts.open_config {
        // Open the config file in the default editor

        let path: PathBuf = opts.config.map_or_else(
            || {
                let home = dirs::home_dir().ok_or_else(|| eyre!("No home directory found"))?;
                color_eyre::eyre::Ok(home.join(Path::new(DEFAULT_CONFIG_PATH)))
            },
            |e| Ok(PathBuf::from(e)),
        )?;
        if !path.exists() {
            // Create the file if it doesn't exist
            tokio::fs::write(&path, "").await?;
            log::debug!("Created config file: {}", path.display());
        }
        log::debug!("Opening config file: {}", path.display());
        utils::open_file(&path)?;
        process::exit(0);
    }
    if opts.default_config {
        // Print the default config to the console
        let default = Opts::default();
        println!("{}", toml::to_string_pretty(&default)?);
        process::exit(0);
    }
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
