pub enum Mode {
    Recursive,
    Permutations,
    Classic,
}

impl From<&str> for Mode {
    fn from(s: &str) -> Self {
        match s {
            "recursive" | "recursion" | "r" => Mode::Recursive,
            "permutations" | "permutation" | "p" => Mode::Permutations,
            "classic" | "c" => Mode::Classic,
            _ => Mode::Recursive,
        }
    }
}
