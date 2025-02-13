use crate::cli::Opts;
use tabled::{settings::Style, Table};

pub fn from_opts(opts: &Opts) -> Table {
    let mut table = Table::new(vec![opts]);

    table.with(Style::modern_rounded());

    table
}
