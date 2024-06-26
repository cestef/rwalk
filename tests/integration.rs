use clap::Parser;
use color_eyre::eyre::Result;
use rwalk::{
    _main,
    cli::opts::{Opts, Wordlist},
};

const SHORT: &str = "tests/wordlists/short.txt";
const EMPTY: &str = "tests/wordlists/empty.txt";

fn opts_from(s: &str) -> Result<Opts, clap::Error> {
    // rwalk <args>
    Opts::try_parse_from(
        vec!["rwalk"]
            .into_iter()
            .chain(s.split(' '))
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>(),
    )
}
#[test]
fn should_parse() {
    assert!(opts_from("http://localhost tests/wordlists/short.txt").is_ok());
}

#[test]
fn error_on_invalid_url_parse() {
    assert!(opts_from("not:an:url_at^all tests/wordlists/short.txt").is_err());
}

#[tokio::test]
async fn error_on_missing_url_main() {
    assert!(_main(Opts {
        wordlists: vec![Wordlist(SHORT.to_string(), vec![])],
        ..Default::default()
    })
    .await
    .is_err())
}

#[tokio::test]
async fn error_on_missing_wordlist_main() {
    assert!(_main(Opts {
        url: Some("http://example.com".to_string()),
        ..Default::default()
    })
    .await
    .is_err())
}

#[tokio::test]
async fn error_on_empty_wordlist_main() {
    assert!(_main(Opts {
        url: Some("http://example.com".to_string()),
        wordlists: vec![Wordlist(EMPTY.to_string(), vec![])],
        ..Default::default()
    })
    .await
    .is_err())
}
