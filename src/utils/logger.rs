use env_logger::{fmt::style::AnsiColor as Color, Builder, Env};

use std::io::Write;

pub fn init_logger() {
    let env = Env::default().filter_or("RWALK_LOG", "info");

    Builder::from_env(env)
        .filter_module("hyper_util::client::legacy::pool", log::LevelFilter::Warn)
        .filter_module("reqwest::connect", log::LevelFilter::Warn)
        .filter_module(
            "hyper_util::client::legacy::connect::http",
            log::LevelFilter::Warn,
        )
        .filter_module(
            "hyper_util::client::legacy::connect::dns",
            log::LevelFilter::Warn,
        )
        .filter_module("rustyline::keymap", log::LevelFilter::Warn)
        .filter_module("rustyline::undo", log::LevelFilter::Warn)
        .filter_module("rustyline::edit", log::LevelFilter::Warn)
        .filter_module("rustyline::tty::unix", log::LevelFilter::Warn)
        .filter_module("rustyline::tty::unix::termios_", log::LevelFilter::Warn)
        .format(|buf, record| {
            let icon_style = env_logger::fmt::style::Style::new().fg_color(Some(
                (match record.level() {
                    log::Level::Info => Color::Blue,
                    log::Level::Warn => Color::Yellow,
                    log::Level::Error => Color::Red,
                    log::Level::Debug => Color::Cyan,
                    log::Level::Trace => Color::Magenta,
                })
                .into(),
            ));

            let icon = match record.level() {
                log::Level::Info => "ℹ",
                log::Level::Warn => "⚠",
                log::Level::Error => "✖",
                log::Level::Debug => "⚙",
                log::Level::Trace => "⚡",
            };

            let module = match record.level() {
                log::Level::Debug => Some(record.module_path().unwrap()),
                log::Level::Trace => Some(record.module_path().unwrap()),
                _ => None,
            };

            let module_style = env_logger::fmt::style::Style::new().dimmed();

            writeln!(
                buf,
                "{icon_style}{}{icon_style:#} {module_style}{}{module_style:#}{}",
                icon,
                if let Some(module) = module {
                    format!("({}) ", module)
                } else {
                    "".to_string()
                },
                record.args()
            )
        })
        .init();
}
