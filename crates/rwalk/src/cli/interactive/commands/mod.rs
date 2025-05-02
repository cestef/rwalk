mod append;
mod clear;
mod eval;
mod exit;
mod get;
mod help;
mod list;
mod run;
mod set;

use crate::cli::Opts;
use crate::utils::registry::create_registry;
use crate::{Result, RwalkError};
use once_cell::sync::Lazy;
use owo_colors::OwoColorize;
use rustyline::history::FileHistory;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::helper::RwalkHelper;

pub struct CommandContext<'a> {
    pub exit: bool,
    pub opts: Opts,
    pub editor: Arc<Mutex<rustyline::Editor<RwalkHelper, FileHistory>>>,
    pub engine: Arc<rhai::Engine>,
    pub scope: Arc<Mutex<rhai::Scope<'a>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum ArgType {
    Path,
    OptionField,
    Url,
    String,
    Int,
    Float,
    Bool,
    Any,
}

#[async_trait::async_trait]
pub trait Command<T>: Debug {
    async fn execute(&self, ctx: &mut T, args: &str) -> Result<()>;
    fn name() -> &'static str
    where
        Self: Sized;
    fn aliases() -> &'static [&'static str]
    where
        Self: Sized,
    {
        &[]
    }

    fn description(&self) -> &'static str;
    fn usage(&self) -> String {
        let mut usage = String::new();
        if let Some(args) = self.args() {
            for arg in args {
                usage.push_str(&format!("<{}> ", arg.blue()));
            }
        }
        usage
    }
    fn construct() -> Box<dyn Command<T>>
    where
        Self: Sized + 'static;

    fn args(&self) -> Option<&'static [ArgType]> {
        None
    }
}

create_registry!(
    command,
    CommandRegistry,
    CommandContext<'static>,
    [
        exit::ExitCommand,
        help::HelpCommand,
        run::RunCommand,
        list::ListCommand,
        set::SetCommand,
        get::GetCommand,
        clear::ClearCommand,
        append::AppendCommand,
        eval::EvalCommand,
    ]
);
