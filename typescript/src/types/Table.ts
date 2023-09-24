// Generated file; do not edit. See `../rust/schema-gen` crate.

import { BlocksOrString } from './BlocksOrString';
import { CreativeWork } from './CreativeWork';
import { TableRow } from './TableRow';

// A table.
export class Table extends CreativeWork {
  type = "Table";

  // A caption for the table.
  caption?: BlocksOrString;

  // A short label for the table.
  label?: string;

  // Rows of cells in the table.
  rows: TableRow[];

  constructor(rows: TableRow[], options?: Table) {
    super()
    if (options) Object.assign(this, options)
    this.rows = rows;
  }

  static from(other: Table): Table {
    return new Table(other.rows!, other)
  }
}
