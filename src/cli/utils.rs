use itertools::Itertools;
use owo_colors::OwoColorize;

use crate::{
    wordlist::{filters::WordlistFilterRegistry, transformation::WordlistTransformerRegistry},
    worker::filters::ResponseFilterRegistry,
};

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

pub fn list_transforms() {
    let transforms = WordlistTransformerRegistry::list();
    println!("{}", "Transforms:".underline());
    for (transform, aliases) in transforms {
        println!(
            " - {} ({})",
            transform.bold(),
            aliases.iter().map(|e| e.dimmed().to_string()).join(", ")
        );
    }
}

pub fn list_filters() {
    let response_filters = ResponseFilterRegistry::list();
    let wordlist_filters = WordlistFilterRegistry::list();
    println!("{}", "Response Filters:".underline());
    for (filter, aliases) in response_filters {
        println!(
            " - {} ({})",
            filter.bold(),
            aliases.iter().map(|e| e.dimmed().to_string()).join(", ")
        );
    }
    println!("\n{}", "Wordlist Filters:".underline());
    for (filter, aliases) in wordlist_filters {
        println!(
            " - {} ({})",
            filter.bold(),
            aliases.iter().map(|e| e.dimmed().to_string()).join(", ")
        );
    }
}
