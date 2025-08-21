use std::path::Path;

use polars::prelude::*;

use codec::common::eyre::Result;

/// Read a CSV file into a Polars [`DataFrame`].
///
/// Uses Polars' lazy CSV reader with header detection enabled for efficient parsing.
/// The lazy evaluation allows for optimization of subsequent operations before
/// materializing the data into memory.
pub fn read_csv(path: &Path) -> Result<DataFrame> {
    let path = PlPath::new(&path.to_string_lossy());
    Ok(LazyCsvReader::new(path)
        .with_has_header(true)
        .finish()?
        .collect()?)
}

/// Write a Polars [`DataFrame`] to a CSV file.
///
/// Creates the target file and writes the DataFrame using Polars' CSV writer.
/// Headers are included by default to maintain data structure information
/// for round-trip compatibility.
pub fn write_csv(df: &DataFrame, path: &Path) -> Result<()> {
    let file = std::fs::File::create(path)?;
    CsvWriter::new(file).finish(&mut df.clone())?;
    Ok(())
}
