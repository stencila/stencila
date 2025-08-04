use std::path::Path;

use calamine::{Reader, Xlsx, open_workbook};

use codec::{
    common::eyre::{Context, Result, bail, eyre},
    schema::Datatable,
};

use crate::conversion::range_to_datatable;

/// Read an XLSX file into a Stencila [`Datatable`].
///
/// Opens the XLSX file using calamine and reads the first worksheet. The first row
/// is treated as column headers, with subsequent rows containing the data. Returns
/// a Datatable with appropriate type validators inferred from the data.
pub fn read_xlsx(path: &Path) -> Result<Datatable> {
    let mut workbook: Xlsx<_> = open_workbook(path).wrap_err("Failed to open XLSX file")?;

    let sheets = workbook.sheet_names();
    let Some(first_sheet) = sheets.first() else {
        bail!("XLSX file contains no worksheets");
    };

    let range = workbook
        .worksheet_range(first_sheet)
        .wrap_err_with(|| eyre!("Failed to read worksheet '{first_sheet}'"))?;

    range_to_datatable(range)
}
