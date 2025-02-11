pub const DEFAULT_RESPONSE_FILTERS: &[(&str, &str)] =
    &[("status", "200-299,301-302,307,401,403,405,500")];
pub const DEFAULT_WORDLIST_KEY: &str = "$";
pub const DEFAULT_THROTTLE_WINDOW_SIZE_MILLIS: u64 = 3000;
pub const DEFAULT_THROTTLE_ERROR_THRESHOLD: f64 = 0.05;

pub const THREADS_PER_CORE: usize = 10;
pub const THROUGHPUT_THRESHOLD: f64 = 0.8; // Require 80% of target RPS

pub const PROGRESS_TEMPLATE: &str = "{spinner:.blue} (ETA. {eta}) [{wide_bar}] {pos:>5}/{len} ({per_sec:>12}) | {prefix:>3} {msg:>14.bold}";
pub const PROGRESS_CHARS: &str = "█▉▊▋▌▍▎▏ ";

pub const STEAL_LIMIT: usize = 16;
