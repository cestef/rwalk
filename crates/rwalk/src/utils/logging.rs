use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() -> miette::Result<()> {
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
                )
                .add_directive(
                    "rustyline=off"
                        .parse()
                        .map_err(|e| miette::miette!("Failed to parse directive: {}", e))?,
                ),
        )
        .init();
    Ok(())
}
