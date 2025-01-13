pub const DEFAULT_RESPONSE_FILTERS: &[(&str, &str)] = &[("status", "200-299")];
pub const THREADS_PER_CORE: usize = 10;
pub const DEFAULT_WORDLIST_KEY: &str = "$";
pub const THROTTLE_WINDOW_SIZE_SEC: u64 = 1;
pub const THROTTLE_ERROR_THRESHOLD: f64 = 0.1;
