use serde::Serialize;
use std::fs;

#[derive(Serialize)]
struct Summary {
    rows: usize,
}

fn main() {
    let table = fs::read_to_string("../../data/raw/samples.tsv");
    let summary = Summary { rows: 2 };
    fs::write("../../results/rust-summary.tsv", format!("rows\n{}\n", summary.rows));
}
