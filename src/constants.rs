use std::ops::RangeInclusive;

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
pub const STATUS_CODES: [RangeInclusive<u16>; 4] = [200..=299, 300..=399, 400..=403, 500..=599];
