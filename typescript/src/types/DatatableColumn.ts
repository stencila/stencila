// Generated file. Do not edit; see `rust/schema-gen` crate.

import { ArrayValidator } from './ArrayValidator';
import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { Primitive } from './Primitive';
import { PropertyValueOrString } from './PropertyValueOrString';
import { String } from './String';

// A column of data within a Datatable.
export class DatatableColumn {
  // The type of this item
  type = "DatatableColumn";

  // The identifier for this item
  id?: String;

  // Alternate names (aliases) for the item.
  alternateNames?: String[];

  // A description of the item.
  description?: Block[];

  // Any kind of identifier for any kind of Thing.
  identifiers?: PropertyValueOrString[];

  // Images of the item.
  images?: ImageObjectOrString[];

  // The name of the item.
  name: String;

  // The URL of the item.
  url?: String;

  // The data values of the column.
  values: Primitive[];

  // The validator to use to validate data in the column.
  validator?: ArrayValidator;

  constructor(name: String, values: Primitive[], options?: DatatableColumn) {
    if (options) Object.assign(this, options)
    this.name = name;
    this.values = values;
  }
}
