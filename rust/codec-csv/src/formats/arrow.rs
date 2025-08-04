use std::path::Path;

use polars::prelude::*;

use codec::common::eyre::Result;

/// Read an Arrow IPC file into a Polars [`DataFrame`].
///
/// Uses Polars' IPC scanner to read Arrow's Inter-Process Communication format.
/// Arrow IPC preserves exact data types and schema information, making it ideal
/// for high-fidelity data exchange between different systems.
pub fn read_arrow(path: &Path) -> Result<DataFrame> {
    Ok(LazyFrame::scan_ipc(path, Default::default())?.collect()?)
}

/// Write a Polars [`DataFrame`] to an Arrow IPC file.
///
/// Uses Polars' IPC writer to create an Arrow Inter-Process Communication file.
/// This format maintains perfect type fidelity and schema information, making it
/// the best choice for lossless data serialization.
pub fn write_arrow(df: &DataFrame, path: &Path) -> Result<()> {
    let file = std::fs::File::create(path)?;
    IpcWriter::new(file).finish(&mut df.clone())?;
    Ok(())
}
