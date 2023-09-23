// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from './Entity';
import { TableCell } from './TableCell';
import { TableRowType } from './TableRowType';

// A row within a Table.
export class TableRow extends Entity {
  type = "TableRow";

  // An array of cells in the row.
  cells: TableCell[];

  // The type of row.
  rowType?: TableRowType;

  constructor(cells: TableCell[], options?: TableRow) {
    super()
    if (options) Object.assign(this, options)
    this.cells = cells;
  }
}
