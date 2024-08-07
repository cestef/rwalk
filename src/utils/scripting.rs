use crate::utils::tree::UrlType;
use crate::utils::tree::{TreeData, TreeNode};
use rhai::plugin::*;
use std::collections::BTreeMap;

use crate::cli::opts::Opts;
use color_eyre::eyre::{eyre, Result};
use colored::Colorize;
use indicatif::ProgressBar;

use reqwest::Response;
use rhai::TypeBuilder;
use rhai::{exported_module, CustomType, Dynamic, Engine, Scope};
use serde::{Deserialize, Serialize};

#[export_module]
pub mod tree_node {

    #[rhai_fn(get = "children")]
    pub fn children(data: &mut TreeNode<TreeData>) -> Dynamic {
        let mut children = Vec::new();
        for child in &data.children {
            children.push(child.lock().clone());
        }
        children.into()
    }

    #[rhai_fn(get = "data")]
    pub fn data(data: &mut TreeNode<TreeData>) -> TreeData {
        data.data.clone()
    }

    #[rhai_fn(global)]
    pub fn to_string(data: &mut TreeNode<TreeData>) -> String {
        format!(
            "TreeNode {{ data: {}, children: [{}] }}",
            tree_data::to_string(&mut data.data),
            data.children.len()
        )
    }
}

#[export_module]
pub mod tree_data {

    #[rhai_fn(global)]
    pub fn to_string(data: &mut TreeData) -> String {
        format!(
            "TreeData {{ url: {}, depth: {}, path: {}, status_code: {}, url_type: {:?}, extra: {:?} }}",
            data.url,
            data.depth,
            data.status_code,
            data.path,
            match &data.url_type {
                UrlType::Directory => "dir",
                UrlType::File(ext) => ext,
                UrlType::Unknown => "unknown",
                UrlType::None => "",
            },
            data.extra
        )
    }
    #[rhai_fn(get = "response")]
    pub fn get_response(data: &mut TreeData) -> Dynamic {
        if let Some(response) = &data.response {
            Dynamic::from(response.clone())
        } else {
            Dynamic::UNIT
        }
    }
}

#[derive(Clone, CustomType, Serialize, Deserialize, Debug, Default)]
pub struct ScriptingResponse {
    pub status_code: u16,
    pub headers: Dynamic,
    pub body: String,
    pub url: String,
}

impl ScriptingResponse {
    pub async fn from_response(response: Response, body: Option<String>) -> Self {
        let headers = response
            .headers()
            .iter()
            .map(|(key, value)| {
                (
                    key.as_str().to_string(),
                    value.to_str().unwrap().to_string(),
                )
            })
            .collect::<BTreeMap<String, String>>();
        ScriptingResponse {
            status_code: response.status().as_u16(),
            headers: headers.into(),
            url: response.url().as_str().to_string(),
            body: if let Some(body) = body {
                body
            } else {
                response.text().await.unwrap_or_default()
            },
        }
    }
}

pub async fn run_scripts(
    opts: &Opts,
    data: &TreeData,
    response: Option<ScriptingResponse>,
    progress: ProgressBar,
) -> Result<()> {
    let mut engine = Engine::new();
    let tree_module = exported_module!(tree_node);
    let tree_data_module = exported_module!(tree_data);

    engine.register_global_module(tree_module.into());
    engine.register_global_module(tree_data_module.into());

    let mut root_scope = Scope::new();
    root_scope.push("data", data.clone());
    root_scope.push("opts", opts.clone());
    root_scope.push("response", response.clone());
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

        let res = engine
            .run_file_with_scope(&mut scope, script.into())
            .map_err(|e| eyre!(format!("Error running script: {}", e)));
        if !opts.ignore_scripts_errors {
            res?;
        }
    }
    Ok(())
}
