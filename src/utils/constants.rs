pub const DEFAULT_RESPONSE_FILTERS: &[(&str, &str)] = &[("status", "200-299")];
pub const DEFAULT_WORDLIST_KEY: &str = "$";
pub const DEFAULT_THROTTLE_WINDOW_SIZE_MILLIS: u64 = 3000;
pub const DEFAULT_THROTTLE_ERROR_THRESHOLD: f64 = 0.05;

pub const THREADS_PER_CORE: usize = 10;
pub const THROUGHPUT_THRESHOLD: f64 = 0.8; // Require 80% of target RPS
