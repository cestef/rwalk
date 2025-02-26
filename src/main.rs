use clap::{CommandFactory, Parser};
use clap_help::Printer;
use merge::Merge;
use rwalk::{
    cli::{utils, Opts},
    run, RwalkError,
};
use termimad::ansi;
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

    if opts.help {
        static INTRO: &str = "
*rwalk* is a web fuzzer that allows you to granularly control the fuzzing process.

It supports:
- fuzzing modes: *r* for `recursive` and *t* for `template`
- filters: *status*, *headers*, ... (see `--list-filters`)
- transforms: *encode*, *case*, ... (see `--list-transforms`)

Complete documentation is available at ~~https://rwalk.cstef.dev~~
";
        static OPTIONS_TEMPLATE: &str = r#"
**Options:**
|:-:|:-:|:-:|
|short|long|description|
|:-:|:-|:-|
${option-lines
|${short}|󠀠${long} *${value-long-braced}* |${help}${possible_values}${default} |
}
|-
"#;

        let mut printer = Printer::new(Opts::command())
            .without("author")
            .with("introduction", INTRO)
            .with("options", OPTIONS_TEMPLATE);
        let skin = printer.skin_mut();
        skin.headers[0].compound_style.set_fg(ansi(217));
        skin.bold.set_fg(ansi(217));
        skin.italic = termimad::CompoundStyle::with_fg(ansi(217));
        skin.strikeout =
            termimad::CompoundStyle::with_attr(termimad::crossterm::style::Attribute::Underdashed);

        skin.table_border_chars = termimad::ROUNDED_TABLE_BORDER_CHARS;
        printer.print_help();
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
