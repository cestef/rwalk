use std::time::Duration;

pub const DEFAULT_RESPONSE_FILTER: &str = "status:200-299,301-302,307,401,403,405,500";
pub const DEFAULT_WORDLIST_KEY: &str = "$";

pub const THREADS_PER_CORE: usize = 10;

pub const PROGRESS_TEMPLATE: &str =
    "{spinner:.blue} (ETA. {eta}) {wide_bar:.blue/dim} {pos:>5}/{len} ({per_sec:>12})";
pub const PROGRESS_CHARS: &str = "█▉▊▋▌▍▎▏·";

pub const STEAL_BATCH_LIMIT: usize = 8;

pub const PROGRESS_UPDATE_INTERVAL: Duration = Duration::from_millis(100);
