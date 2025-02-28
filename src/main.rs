use clap::Parser;

use merge::Merge;
use rwalk::{
    cli::{help, utils, Opts},
    run, RwalkError,
};

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

    if opts.help || opts.help_long {
        help::print(opts.help_long);
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
