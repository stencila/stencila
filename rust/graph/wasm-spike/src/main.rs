use serde::Deserialize;
use std::io::{self, BufRead};

#[derive(Deserialize)]
struct Input {
    language: String,
    source: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for line in io::stdin().lock().lines() {
        let input: Input = serde_json::from_str(&line?)?;
        let output = stencila_code_analysis_spike::analyze(&input.language, &input.source)?;
        println!("{output}");
    }
    Ok(())
}
