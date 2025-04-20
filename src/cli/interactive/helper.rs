use rustyline::{
    completion::Completer, highlight::Highlighter, hint::Hinter, validate::Validator, Helper,
};

pub struct RwalkHelper;

impl Validator for RwalkHelper {}

impl Highlighter for RwalkHelper {}
impl Hinter for RwalkHelper {
    type Hint = String;
}

impl Completer for RwalkHelper {
    type Candidate = String;
}

impl Helper for RwalkHelper {}
