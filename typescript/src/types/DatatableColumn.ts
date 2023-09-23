// Generated file; do not edit. See `../rust/schema-gen` crate.

import { ArrayValidator } from './ArrayValidator';
import { Primitive } from './Primitive';
import { Thing } from './Thing';

// A column of data within a Datatable.
export class DatatableColumn extends Thing {
  type = "DatatableColumn";

  // The data values of the column.
  values: Primitive[];

  // The validator to use to validate data in the column.
  validator?: ArrayValidator;

  constructor(name: string, values: Primitive[], options?: DatatableColumn) {
    super()
    if (options) Object.assign(this, options)
    this.name = name;
    this.values = values;
  }
}
