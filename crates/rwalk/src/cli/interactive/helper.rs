use owo_colors::OwoColorize;
use rustyline::{
    Helper, completion::Completer, highlight::Highlighter, hint::Hinter, validate::Validator,
};
use termimad::crossterm::style::Stylize;
use tracing::debug;

use crate::cli::interactive::commands::ArgType;

use super::commands::CommandRegistry;

pub struct RwalkHelper;

impl Validator for RwalkHelper {}

impl Highlighter for RwalkHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        debug!("Highlighting line: {}", line);
        let command_end = line.find(' ').unwrap_or(line.len());
        let cmd = &line[..command_end];
        let arg = &line[command_end..];

        let highlighted_cmd = if CommandRegistry::exists(cmd) {
            cmd.green()
        } else {
            cmd.red()
        };

        let highlighted_line = format!("{}{}", highlighted_cmd.bold(), arg);

        highlighted_line.into()
    }

    fn highlight_char(
        &self,
        _line: &str,
        _pos: usize,
        _kind: rustyline::highlight::CmdKind,
    ) -> bool {
        true
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str, // FIXME should be Completer::Candidate
        _completion: rustyline::CompletionType,
    ) -> std::borrow::Cow<'c, str> {
        candidate.dimmed().to_string().into()
    }
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> std::borrow::Cow<'b, str> {
        if default {
            format!("\x1b[1;34m{}\x1b[0m", prompt).into()
        } else {
            format!("\x1b[1;32m{}\x1b[0m", prompt).into()
        }
    }
}
impl Hinter for RwalkHelper {
    type Hint = String;
}

impl Completer for RwalkHelper {
    type Candidate = String;

    fn complete(
        &self, // FIXME should be `&mut self`
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let available_commands = CommandRegistry::list();
        // Check if pos is at the command or the argument part
        let command_end = line.find(' ').unwrap_or(line.len());
        let cmd = &line[..command_end];
        let is_completing_command = pos <= command_end;
        debug!(
            "is_completing_command: {}, cmd: {}",
            is_completing_command, cmd
        );

        if is_completing_command {
            let mut completions = Vec::new();
            for (available_cmd, aliases) in available_commands {
                if available_cmd.starts_with(cmd) {
                    completions.push(available_cmd.to_string());
                }
                for alias in aliases {
                    if alias.starts_with(cmd) {
                        completions.push(alias.to_string());
                    }
                }
            }
            return Ok((0, completions));
        } else {
            let arg_start = line[..pos].rfind(' ').unwrap_or(command_end);
            let arg = &line[arg_start + 1..pos];
            // e.g. run localhost common.txt: localhost=0, common.txt=1
            let arg_index = line[..arg_start].split_whitespace().count() - 1;
            if let Ok(cmd) = CommandRegistry::construct(cmd) {
                if let Some(args) = cmd.args() {
                    if args.is_empty() {
                        return Ok((pos, vec![]));
                    }

                    if arg_index >= args.len() {
                        return Ok((pos, vec![]));
                    }
                    let arg_type = args[arg_index];
                    debug!("arg_type: {:?}", arg_type);

                    match arg_type {
                        ArgType::Path => complete_path(arg, arg_start),
                        _ => Ok((pos, vec![])),
                    }
                } else {
                    Ok((pos, vec![]))
                }
            } else {
                debug!("Command not found: {}", cmd);
                Ok((pos, vec![]))
            }
        }
    }
}

impl Helper for RwalkHelper {}

fn complete_path(arg: &str, arg_start: usize) -> rustyline::Result<(usize, Vec<String>)> {
    let mut completions = Vec::new();
    let path = std::path::Path::new(arg);
    let (dir_to_search, file_prefix) =
        if arg.is_empty() || arg.ends_with('/') || arg.ends_with('\\') {
            // Complete inside the directory
            (std::path::PathBuf::from(arg), String::new())
        } else {
            // Complete based on prefix
            match path.parent() {
                Some(parent) => {
                    let parent_path = if parent.as_os_str().is_empty() {
                        std::path::PathBuf::from(".")
                    } else {
                        parent.to_path_buf()
                    };

                    let filename = path
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_default();

                    (parent_path, filename)
                }
                None => (std::path::PathBuf::from("."), arg.to_string()),
            }
        };

    debug!(
        "Searching dir: {:?}, prefix: {}",
        dir_to_search, file_prefix
    );

    // Read the directory and find matching entries
    if let Ok(entries) = std::fs::read_dir(&dir_to_search) {
        for entry in entries.filter_map(Result::ok) {
            let entry_name = entry.file_name();
            let entry_name_str = entry_name.to_string_lossy();

            if entry_name_str.starts_with(&file_prefix) {
                // Create the full path for completion
                let mut completion = if dir_to_search.as_os_str().is_empty()
                    || dir_to_search == std::path::Path::new(".")
                {
                    entry_name_str.to_string()
                } else {
                    let mut path_string = dir_to_search.to_string_lossy().to_string();
                    if !path_string.ends_with('/') && !path_string.ends_with('\\') {
                        path_string.push('/');
                    }
                    path_string + entry_name_str.as_ref()
                };

                // Add a trailing slash for directories
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    completion.push('/');
                }

                completions.push(completion);
            }
        }
    }

    // Sort completions for better user experience
    completions.sort();

    Ok((arg_start + 1, completions))
}
