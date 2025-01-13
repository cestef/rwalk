use clap::Parser;

use rwalk::{cli::Opts, run};
use tracing::Level;
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
                    "hyper_util::client::legacy::pool=off"
                        .parse()
                        .map_err(|_| miette::miette!("Failed to parse directive"))?,
                )
                .add_directive(Level::INFO.into()),
        )
        .init();

    let opts = Opts::parse();
    run(opts).await?;
    Ok(())
}
