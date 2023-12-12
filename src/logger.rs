use env_logger::{fmt::Color, Builder, Env};

use std::io::Write;

pub fn init_logger() {
    let env = Env::default().filter_or("RWALK_LOG", "info");

    Builder::from_env(env)
        .format(|buf, record| {
            let mut style = buf.style();
            match record.level() {
                log::Level::Info => style.set_color(Color::Green),
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

            writeln!(buf, "{} {}", icon, record.args())
        })
        .init();
}
