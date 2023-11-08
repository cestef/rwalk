use colored::Colorize;

use crate::ARGS;

pub fn log_info(message: &str, clear: bool) {
    if clear {
        print!("\r\x1b[K");
    }
    println!("{} {}", "ℹ".cyan(), message);
}

pub fn log_error(message: &str, clear: bool) {
    if clear {
        print!("\r\x1b[K");
    }
    println!("{} {}", "✖".red(), message);
}

pub fn log_success(message: &str, clear: bool) {
    if clear {
        print!("\r\x1b[K");
    }
    println!("{} {}", "✓".green(), message);
}

pub fn log_warning(message: &str, clear: bool) {
    if ARGS.verbose > 0 {
        if clear {
            print!("\r\x1b[K");
        }
        println!("{} {}", "⚠".yellow(), message);
    }
}

pub fn log(message: &str, clear: bool) {
    if clear {
        print!("\r\x1b[K");
    }
    println!("{}", message);
}

pub fn log_verbose(message: &str, clear: bool) {
    if ARGS.verbose > 1 {
        if clear {
            print!("\r\x1b[K");
        }
        println!("{}", message);
    }
}
