use clap::Parser;

use clap_markdown::MarkdownOptions;
use merge::Merge;
use rwalk::{
    RwalkError,
    cli::{Opts, help, interactive, utils},
    run,
    utils::types::ListType,
};

use tracing::debug;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

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
                )
                .add_directive(
                    "rustyline=off"
                        .parse()
                        .map_err(|e| miette::miette!("Failed to parse directive: {}", e))?,
                ),
        )
        .init();

    let mut opts = Opts::parse();
    debug!("{:#?}", opts);

    if opts.help || opts.help_long {
        help::print(opts.help_long);
        return Ok(());
    }

    if opts.generate_markdown {
        let mut markdown = clap_markdown::help_markdown_custom::<Opts>(
            &MarkdownOptions::new()
                .show_footer(false)
                .show_table_of_contents(false),
        );
        markdown = markdown
            .split_at(markdown.find("**Usage:**").unwrap_or(0))
            .1
            .to_string();
        const FRONTMATTER: &str = "+++\ntitle = \"Options\"\nweight = 1000\n+++\n";
        print!("{FRONTMATTER}\n{markdown}");
        return Ok(());
    }

    if let Some(list) = opts.list {
        match list {
            ListType::Filters => utils::list_filters(),
            ListType::Transforms => utils::list_transforms(),
            ListType::All => {
                utils::list_filters();
                println!();
                utils::list_transforms();
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
