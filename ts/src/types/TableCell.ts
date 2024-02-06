// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Entity } from "./Entity.js";
import { Integer } from "./Integer.js";
import { TableCellType } from "./TableCellType.js";

/**
 * A cell within a `Table`.
 */
export class TableCell extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "TableCell";

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
  content: Block[];

  constructor(content: Block[], options?: Partial<TableCell>) {
    super();
    this.type = "TableCell";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `TableCell`
*/
export function tableCell(content: Block[], options?: Partial<TableCell>): TableCell {
  return new TableCell(content, options);
}
