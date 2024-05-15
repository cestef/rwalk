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
    let mut root_scope = Scope::new();
    root_scope.push("data", data.clone());
    root_scope.push("opts", opts.clone());
    let engine_progress = progress.clone();
    let engine_opts = opts.clone();
    engine.on_print(move |s| {
        if !engine_opts.quiet {
            engine_progress.println(s);
        }
    });
    for script in &opts.scripts {
        if !opts.quiet {
            progress.println(format!(
                "{} Running script: {}",
                "→".dimmed(),
                script.dimmed()
            ));
        }
        let mut scope = root_scope.clone();

        engine
            .run_file_with_scope(&mut scope, script.into())
            .map_err(|e| anyhow!(format!("Error running script: {}", e)))?;
    }
    Ok(())
}
