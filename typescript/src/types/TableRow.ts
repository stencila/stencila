// Generated file; do not edit. See `../rust/schema-gen` crate.

import { String } from './String';
import { TableCell } from './TableCell';
import { TableRowType } from './TableRowType';

// A row within a Table.
export class TableRow {
  // The type of this item
  type = "TableRow";

  // The identifier for this item
  id?: String;

  // An array of cells in the row.
  cells: TableCell[];

  // The type of row.
  rowType?: TableRowType;

  constructor(cells: TableCell[], options?: TableRow) {
    if (options) Object.assign(this, options)
    this.cells = cells;
  }
}
