use clap::Parser;

use merge::Merge;
use rwalk::{
    RwalkError,
    cli::{self, Opts, help, interactive},
    run,
    utils::{self, types::ListType},
};

use tracing::debug;

#[tokio::main]
async fn main() -> miette::Result<()> {
    utils::logging::init()?;

    let mut opts = Opts::parse();
    debug!("{:#?}", opts);

    if opts.help || opts.help_long {
        help::print(opts.help_long);
        return Ok(());
    }

    if opts.generate_markdown {
        utils::markdown::generate_markdown();
        return Ok(());
    }

    if let Some(list) = opts.list {
        match list {
            ListType::Filters => cli::utils::list_filters(),
            ListType::Transforms => cli::utils::list_transforms(),
            ListType::All => {
                cli::utils::list_filters();
                println!();
                cli::utils::list_transforms();
            }
        }
        return Ok(());
    }

    if let Some(ref config) = opts.config {
        let config = tokio::fs::read_to_string(config)
            .await
            .map_err(RwalkError::from)?;
        let config: Opts = toml::from_str(&config).map_err(RwalkError::from)?;
        opts.merge(config);
        debug!("merged: {:#?}", opts);
    }

    if opts.interactive {
        opts.interactive = false;
        interactive::run(opts).await?
    } else {
        run(opts, None).await?
    };
    Ok(())
}
