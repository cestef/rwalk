use std::time::Duration;

pub const DEFAULT_RESPONSE_FILTER: &str = "status:200-299,301-302,307,401,403,405,500";
pub const DEFAULT_WORDLIST_KEY: &str = "$";

pub const THREADS_PER_CORE: usize = 5;

pub const PROGRESS_TEMPLATE: &str =
    "{spinner:.blue} (ETA. {eta}) {wide_bar:.blue/dim} {pos:>5}/{len} ({per_sec:>12}) {msg:.bold}";
pub const PROGRESS_CHARS: &str = "█▉▊▋▌▍▎▏·";

pub const STEAL_BATCH_LIMIT: usize = 8;

pub const PROGRESS_UPDATE_INTERVAL: Duration = Duration::from_millis(100);

pub const HISTORY_FILE: &str = ".rwalk_history";
pub const STATE_FILE: &str = ".rwalk_state";

pub const RESULTS_VAR_RHAI: &str = "res";
