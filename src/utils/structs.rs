pub enum Mode {
    Recursive,
    Classic,
}

impl From<&str> for Mode {
    fn from(s: &str) -> Self {
        match s {
            "recursive" | "recursion" | "r" => Mode::Recursive,
            "classic" | "c" => Mode::Classic,
            _ => Mode::Recursive,
        }
    }
}
