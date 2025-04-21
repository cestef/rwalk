mod exit;
mod help;
mod list;
mod run;
mod set;

use crate::cli::Opts;
use crate::utils::registry::create_registry;
use crate::{Result, RwalkError};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt::Debug;

pub struct CommandContext {
    pub exit: bool,
    pub opts: Opts,
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

    fn help(&self) -> &'static str;

    fn construct() -> Box<dyn Command<T>>
    where
        Self: Sized + 'static;
}

create_registry!(
    command,
    CommandRegistry,
    CommandContext,
    [
        exit::ExitCommand,
        help::HelpCommand,
        run::RunCommand,
        list::ListCommand,
        set::SetCommand
    ]
);
