use owo_colors::OwoColorize;

pub fn version() -> String {
    let ver = env!("CARGO_PKG_VERSION");
    format!("v{}", ver).bold().to_string()
}
pub fn long_version() -> String {
    let author = env!("CARGO_PKG_AUTHORS");

    let author = author.replace(':', ", ").dimmed().bold().to_string();

    let hash = env!("_GIT_INFO").bold();
    format!(
        "\
{hash}

Authors: {author}"
    )
}
