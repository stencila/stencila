use std::{
    error::Error,
    io::{self, BufRead},
};

fn main() -> Result<(), Box<dyn Error>> {
    for line in io::stdin().lock().lines() {
        println!("{}", code_analysis_wasm_smoke::analyze_json(&line?)?);
    }
    Ok(())
}
