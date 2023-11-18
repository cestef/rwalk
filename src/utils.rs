use colored::Colorize;
use std::io::{Read, Write};

use crate::constants::BANNER_STR;

pub fn parse_wordlists(wordlists: &Vec<String>) -> Vec<String> {
    let mut wordlist = Vec::new();
    for wordlist_path in wordlists {
        let mut file = std::fs::File::open(wordlist_path).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();

        let contents = unsafe { String::from_utf8_unchecked(bytes) };

        for word in contents.split("\n") {
            let word = word.trim();
            if word.len() > 0 {
                wordlist.push(word.to_string());
            }
        }
    }
    wordlist
}

pub fn banner() {
    println!("{}", BANNER_STR.to_string().bold().bright_red());
    println!(
        "{} {}",
        "Version:".dimmed(),
        env!("CARGO_PKG_VERSION").dimmed().bold()
    );
    println!("{} {}", "Author:".dimmed(), "cstef".dimmed().bold());
    println!("");
}

pub fn hide_cursor() {
    print!("\x1B[?25l");
    std::io::stdout().flush().unwrap();
}

pub fn show_cursor() {
    print!("\x1B[?25h");
    std::io::stdout().flush().unwrap();
}
