use std::{collections::HashMap, sync::Arc};

use indicatif::{MultiProgress, ProgressBar};
use lazy_static::lazy_static;
use parking_lot::Mutex;

lazy_static! {
    pub static ref PROGRESS: MultiProgress = MultiProgress::new();
    pub static ref PROGRESSES: Arc<Mutex<HashMap<String, ProgressBar>>> =
        Arc::new(Mutex::new(HashMap::new()));
}
