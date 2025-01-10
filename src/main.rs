use clap::Parser;

use rwalk::{cli::Opts, run};

#[tokio::main]
async fn main() -> miette::Result<()> {
    let opts = Opts::parse();
    run(opts).await?;
    Ok(())
}
