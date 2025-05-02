use itertools::Itertools;
use owo_colors::OwoColorize;
use serde_json::Value;

use super::{Command, CommandContext};
use crate::Result;

#[derive(Debug)]
pub struct ListCommand;
#[async_trait::async_trait]
impl<'a> Command<CommandContext<'a>> for ListCommand {
    async fn execute(&self, ctx: &mut CommandContext, _args: &str) -> Result<()> {
        let fields = ctx.opts.as_nested_map();
        let max_key_len = fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);

        for (key, value) in fields.iter().sorted_by(|a, b| a.0.cmp(b.0)) {
            println!(
                "{} {dots} = {}",
                key.bold(),
                highlight_json(&value),
                dots = "·".repeat(max_key_len - key.len()).dimmed(),
            );
        }
        Ok(())
    }

    fn name() -> &'static str {
        "list"
    }

    fn aliases() -> &'static [&'static str] {
        &["options", "ls"]
    }

    fn description(&self) -> &'static str {
        "List current options"
    }

    fn construct() -> Box<dyn Command<CommandContext<'a>>>
    where
        Self: Sized + 'static,
    {
        Box::new(ListCommand)
    }
}
fn highlight_json(value: &Value) -> String {
    match value {
        Value::Null => "null".dimmed().to_string(),
        Value::Bool(b) => {
            if *b {
                "true".green().to_string()
            } else {
                "false".red().to_string()
            }
        }
        Value::Number(n) => n.to_string().bright_yellow().to_string(),
        Value::String(s) => format!("\"{}\"", s).bright_green().to_string(),
        Value::Array(arr) => {
            let mut result = "[".blue().to_string();

            for (i, item) in arr.iter().enumerate() {
                result.push_str(&highlight_json(item));
                if i < arr.len() - 1 {
                    result.push_str(", ");
                }
            }

            result.push_str(&"]".blue().to_string());
            result
        }
        Value::Object(obj) => {
            let mut result = "{".blue().to_string();

            let keys: Vec<&String> = obj.keys().collect();
            for (i, key) in keys.iter().enumerate() {
                result.push_str(&format!("{}: ", format!("\"{}\"", key).cyan()));
                result.push_str(&highlight_json(&obj[*key]));

                if i < keys.len() - 1 {
                    result.push_str(", ");
                }
            }

            result.push_str(&"}".blue().to_string());
            result
        }
    }
}
