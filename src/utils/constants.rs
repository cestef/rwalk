pub const DEFAULT_RESPONSE_FILTERS: &[(&str, &str)] = &[("status", "200-299")];
pub const DEFAULT_WORDLIST_KEY: &str = "$";
pub const DEFAULT_THROTTLE_WINDOW_SIZE_SEC: u64 = 600;
pub const DEFAULT_THROTTLE_ERROR_THRESHOLD: f64 = 0.3;

pub const THREADS_PER_CORE: usize = 10;
