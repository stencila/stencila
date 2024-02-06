// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { TableCell } from "./TableCell.js";
import { TableRowType } from "./TableRowType.js";

/**
 * A row within a Table.
 */
export class TableRow extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "TableRow";

  /**
   * An array of cells in the row.
   */
  cells: TableCell[];

  /**
   * The type of row.
   */
  rowType?: TableRowType;

  constructor(cells: TableCell[], options?: Partial<TableRow>) {
    super();
    this.type = "TableRow";
    if (options) Object.assign(this, options);
    this.cells = cells;
  }
}

/**
* Create a new `TableRow`
*/
export function tableRow(cells: TableCell[], options?: Partial<TableRow>): TableRow {
  return new TableRow(cells, options);
}
