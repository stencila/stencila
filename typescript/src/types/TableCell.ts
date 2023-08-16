// Generated file. Do not edit; see `rust/schema-gen` crate.

import { BlocksOrInlines } from './BlocksOrInlines';
import { Integer } from './Integer';
import { String } from './String';
import { TableCellType } from './TableCellType';

// A cell within a `Table`.
export class TableCell {
  // The type of this item
  type = "TableCell";

  // The identifier for this item
  id?: String;

  // The name of the cell.
  name?: String;

  // How many columns the cell extends.
  colspan?: Integer;

  // The type of cell.
  cellType?: TableCellType;

  // How many columns the cell extends.
  rowspan?: Integer;

  // Contents of the table cell.
  content?: BlocksOrInlines;

  constructor(options?: TableCell) {
    if (options) Object.assign(this, options)
    
  }
}
