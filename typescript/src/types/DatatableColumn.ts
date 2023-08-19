// Generated file; do not edit. See `../rust/schema-gen` crate.

import { ArrayValidator } from './ArrayValidator';
import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { Primitive } from './Primitive';
import { PropertyValueOrString } from './PropertyValueOrString';

// A column of data within a Datatable.
export class DatatableColumn {
  type = "DatatableColumn";

  // The identifier for this item
  id?: string;

  // Alternate names (aliases) for the item.
  alternateNames?: string[];

  // A description of the item.
  description?: Block[];

  // Any kind of identifier for any kind of Thing.
  identifiers?: PropertyValueOrString[];

  // Images of the item.
  images?: ImageObjectOrString[];

  // The name of the item.
  name: string;

  // The URL of the item.
  url?: string;

  // The data values of the column.
  values: Primitive[];

  // The validator to use to validate data in the column.
  validator?: ArrayValidator;

  constructor(name: string, values: Primitive[], options?: DatatableColumn) {
    if (options) Object.assign(this, options)
    this.name = name;
    this.values = values;
  }
}
