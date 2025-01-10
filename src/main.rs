use clap::Parser;
use eyre::Result;

use rwalk::{cli::Opts, run};

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    run(opts).await
}
