use anyhow::Result;
use clap::Parser;
use rwalk::{_main, cli::opts::Opts};

const SHORT: &str = "tests/wordlists/short.txt";
const EMPTY: &str = "tests/wordlists/empty.txt";

fn opts_from(s: &str) -> Result<Opts, clap::Error> {
    // rwalk <args>
    Opts::try_parse_from(
        vec!["rwalk"]
            .into_iter()
            .chain(s.split(" "))
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>(),
    )
}

#[test]
fn error_on_empty_parse() {
    assert!(opts_from("").is_err());
}

#[test]
fn error_on_missing_wordlist_parse() {
    assert!(opts_from("http://localhost").is_err());
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
        wordlists: vec![SHORT.to_string()],
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
        wordlists: vec![EMPTY.to_string()],
        ..Default::default()
    })
    .await
    .is_err())
}

#[tokio::test]
async fn run_default_arguments() {
    let res = _main(Opts {
        url: Some("http://example.com".to_string()),
        wordlists: vec![SHORT.to_string()],
        ..Default::default()
    })
    .await;

    if res.is_err() {
        panic!("{}", res.unwrap_err())
    }
}

#[tokio::test]
async fn run_custom_arguments() {
    let res = _main(Opts {
        url: Some("http://example.com".to_string()),
        wordlists: vec![SHORT.to_string()],
        method: Some("POST".to_string()),
        data: Some("test".to_string()),
        headers: vec!["Content-Type: application/json".to_string()],
        ..Default::default()
    })
    .await;

    if res.is_err() {
        panic!("{}", res.unwrap_err())
    }
}
