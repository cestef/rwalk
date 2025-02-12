use clap::Parser;

use indicatif::HumanDuration;
use owo_colors::OwoColorize;
use rwalk::{cli::Opts, run};
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

    let opts = Opts::parse();
    debug!("{:#?}", opts);

    let start = std::time::Instant::now();

    let rate = run(opts).await?;

    println!(
        "Done in {} with an average of {} req/s",
        format!("{:#}", HumanDuration(start.elapsed())).bold(),
        rate.round().bold()
    );

    Ok(())
}
