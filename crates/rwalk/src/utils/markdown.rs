use clap_markdown::MarkdownOptions;

use crate::cli::Opts;

pub fn generate_markdown() -> () {
    let mut markdown = clap_markdown::help_markdown_custom::<Opts>(
        &MarkdownOptions::new()
            .show_footer(false)
            .show_table_of_contents(false),
    );
    markdown = markdown
        .split_at(markdown.find("**Usage:**").unwrap_or(0))
        .1
        .to_string();
    const FRONTMATTER: &str = "+++\ntitle = \"Options\"\nweight = 1000\n+++\n";
    print!("{FRONTMATTER}\n{markdown}");
}
