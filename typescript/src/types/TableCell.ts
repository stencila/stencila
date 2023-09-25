// Generated file; do not edit. See `../rust/schema-gen` crate.

import { BlocksOrInlines } from "./BlocksOrInlines.js";
import { Entity } from "./Entity.js";
import { Integer } from "./Integer.js";
import { TableCellType } from "./TableCellType.js";

/**
 * A cell within a `Table`.
 */
export class TableCell extends Entity {
  type = "TableCell";

  /**
   * The type of cell.
   */
  cellType?: TableCellType;

  /**
   * The name of the cell.
   */
  name?: string;

  /**
   * How many columns the cell extends.
   */
  columnSpan?: Integer;

  /**
   * How many columns the cell extends.
   */
  rowSpan?: Integer;

  /**
   * Contents of the table cell.
   */
  content?: BlocksOrInlines;

  constructor(options?: Partial<TableCell>) {
    super();
    if (options) Object.assign(this, options);
    
  }

  /**
  * Create a `TableCell` from an object
  */
  static from(other: TableCell): TableCell {
    return new TableCell(other);
  }
}

/**
* Create a new `TableCell`
*/
export function tableCell(options?: Partial<TableCell>): TableCell {
  return new TableCell(options);
}
