use std::path::Path;

use calamine::{Reader, Xls, open_workbook};

use codec::{
    eyre::{Context, Result, bail, eyre},
    schema::Datatable,
};

use crate::conversion::range_to_datatable;

/// Read an XLS file into a Stencila [`Datatable`].
///
/// Opens the legacy Excel XLS file using calamine and reads the first worksheet.
/// The first row is treated as column headers, with subsequent rows containing the data.
/// Returns a Datatable with appropriate type validators inferred from the data.
pub fn read_xls(path: &Path) -> Result<Datatable> {
    let mut workbook: Xls<_> = open_workbook(path).wrap_err("Failed to open XLS file")?;

    let sheets = workbook.sheet_names();
    let Some(first_sheet) = sheets.first() else {
        bail!("XLS file contains no worksheets");
    };

    let range = workbook
        .worksheet_range(first_sheet)
        .wrap_err_with(|| eyre!("Failed to read worksheet '{first_sheet}'"))?;

    range_to_datatable(range)
}
