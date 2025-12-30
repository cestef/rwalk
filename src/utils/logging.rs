use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() -> miette::Result<()> {
    let indicatif_layer = IndicatifLayer::new();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .with({
            let mut filter = EnvFilter::from_env("RWALK_LOG");
            for directive in ["hyper_util", "reqwest", "rustyline", "mio"] {
                let directive = format!("{}=off", directive)
                    .parse()
                    .map_err(|e| miette::miette!("Failed to parse directive: {}", e))?;
                filter = filter.add_directive(directive);
            }
            filter
        })
        .init();
    Ok(())
}
