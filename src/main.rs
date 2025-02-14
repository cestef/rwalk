use clap::Parser;
use indicatif::HumanDuration;
use itertools::Itertools;
use merge::Merge;
use owo_colors::OwoColorize;
use rwalk::{
    cli::Opts,
    run,
    wordlist::{filters::WordlistFilterRegistry, transformation::WordlistTransformerRegistry},
    worker::filters::ResponseFilterRegistry,
    RwalkError,
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

    if opts.list_filters {
        let response_filters = ResponseFilterRegistry::list();
        let wordlist_filters = WordlistFilterRegistry::list();
        println!("{}", "Response Filters:".underline());
        for (filter, aliases) in response_filters {
            println!(
                " - {} ({})",
                filter.bold(),
                aliases.iter().map(|e| e.dimmed().to_string()).join(", ")
            );
        }
        println!("\n{}", "Wordlist Filters:".underline());
        for (filter, aliases) in wordlist_filters {
            println!(
                " - {} ({})",
                filter.bold(),
                aliases.iter().map(|e| e.dimmed().to_string()).join(", ")
            );
        }

        return Ok(());
    }

    if opts.list_transforms {
        let transforms = WordlistTransformerRegistry::list();
        println!("{}", "Transforms:".underline());
        for (transform, aliases) in transforms {
            println!(
                " - {} ({})",
                transform.bold(),
                aliases.iter().map(|e| e.dimmed().to_string()).join(", ")
            );
        }

        return Ok(());
    }

    let start = std::time::Instant::now();

    let rate = run(opts).await?;

    println!(
        "Done in {} with an average of {} req/s",
        format!("{:#}", HumanDuration(start.elapsed())).bold(),
        rate.round().bold()
    );

    Ok(())
}
