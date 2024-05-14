use std::sync::Arc;

use crate::{
    cli::opts::Opts,
    utils::tree::{tree, TreeData},
};
use anyhow::{anyhow, Result};
use colored::Colorize;
use indicatif::ProgressBar;

use rhai::{exported_module, Engine, Scope};

pub async fn run_scripts(opts: &Opts, data: &TreeData, progress: ProgressBar) -> Result<()> {
    let mut engine = Engine::new();
    let tree_module = exported_module!(tree);

    engine.register_global_module(tree_module.into());
    for script in &opts.scripts {
        progress.println(format!(
            "{} Running script: {}",
            "→".dimmed(),
            script.dimmed()
        ));
        let mut scope = Scope::new();
        scope.push("data", data.clone());
        scope.push("opts", opts.clone());
        let progress = Arc::new(progress.clone());
        engine.on_print(move |s| {
            progress.println(s);
        });
        engine
            .run_file_with_scope(&mut scope, script.into())
            .map_err(|e| anyhow!(format!("Error running script: {}", e)))?;
    }
    Ok(())
}
