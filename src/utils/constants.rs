pub const SUCCESS: char = '✓';
pub const ERROR: char = '✖';
pub const WARNING: char = '⚠';
pub const INFO: char = 'ℹ';
pub const BANNER_STR: &str = r#"
                    _ _    
                   | | |   
 _ ____      ____ _| | | __
| '__\ \ /\ / / _` | | |/ /
| |   \ V  V / (_| | |   < 
|_|    \_/\_/ \__,_|_|_|\_\   
"#;
pub const SAVE_FILE: &str = ".rwalk.json";
pub const STATUS_CODES: &str = "200-299,301,302,307,401,403,405,500";
pub const PROGRESS_TEMPLATE: &str = "{spinner:.blue} (ETA. {eta}) [{wide_bar}] {pos:>5}/{len} ({per_sec:>12}) | {prefix:>3} {msg:>14.bold}";
pub const PROGRESS_CHARS: &str = "█▉▊▋▌▍▎▏ ";
pub const FUZZ_KEY: &str = "$";
