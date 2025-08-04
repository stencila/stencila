use std::path::Path;

use polars::prelude::*;

use codec::common::eyre::Result;

/// Read a TSV (Tab-Separated Values) file into a Polars [`DataFrame`].
///
/// Uses Polars' lazy CSV reader configured with tab separator for efficient parsing.
/// Like CSV reading, this uses lazy evaluation to optimize subsequent operations
/// before materializing the data.
pub fn read_tsv(path: &Path) -> Result<DataFrame> {
    Ok(LazyCsvReader::new(path)
        .with_has_header(true)
        .with_separator(b'\t')
        .finish()?
        .collect()?)
}

/// Write a Polars [`DataFrame`] to a TSV (Tab-Separated Values) file.
///
/// Uses Polars' CSV writer configured with tab separator instead of commas.
/// Headers are included to maintain compatibility with the TSV format standards
/// and enable proper round-trip conversion.
pub fn write_tsv(df: &DataFrame, path: &Path) -> Result<()> {
    let file = std::fs::File::create(path)?;
    CsvWriter::new(file)
        .with_separator(b'\t')
        .finish(&mut df.clone())?;
    Ok(())
}
