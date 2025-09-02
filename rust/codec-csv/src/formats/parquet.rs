use std::path::Path;

use polars::prelude::*;

use stencila_codec::eyre::Result;

/// Read a Parquet file into a Polars [`DataFrame`].
///
/// Uses Polars' lazy Parquet scanner for efficient columnar data reading.
/// Parquet's columnar format allows for optimal memory usage and fast
/// queries, especially when only specific columns are needed.
pub fn read_parquet(path: &Path) -> Result<DataFrame> {
    let path = PlPath::new(&path.to_string_lossy());
    Ok(LazyFrame::scan_parquet(path, Default::default())?.collect()?)
}

/// Write a Polars [`DataFrame`] to a Parquet file.
///
/// Uses Polars' Parquet writer to create a compressed columnar file.
/// Parquet format provides excellent compression ratios and preserves
/// data types more precisely than text-based formats like CSV.
pub fn write_parquet(df: &DataFrame, path: &Path) -> Result<()> {
    let file = std::fs::File::create(path)?;
    ParquetWriter::new(file).finish(&mut df.clone())?;
    Ok(())
}
