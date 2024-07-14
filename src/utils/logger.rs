use env_logger::{fmt::Color, Builder, Env};

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
            let mut style = buf.style();
            match record.level() {
                log::Level::Info => style.set_color(Color::Blue),
                log::Level::Warn => style.set_color(Color::Yellow),
                log::Level::Error => style.set_color(Color::Red),
                log::Level::Debug => style.set_color(Color::Cyan),
                log::Level::Trace => style.set_color(Color::Magenta),
            };

            let icon = style.value(match record.level() {
                log::Level::Info => "ℹ",
                log::Level::Warn => "⚠",
                log::Level::Error => "✖",
                log::Level::Debug => "⚙",
                log::Level::Trace => "⚡",
            });
            let module = match record.level() {
                log::Level::Debug => Some(style.value(record.module_path().unwrap())),
                log::Level::Trace => Some(style.value(record.module_path().unwrap())),
                _ => None,
            };

            writeln!(
                buf,
                "{} {}{}",
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
