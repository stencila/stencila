// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { BlocksOrInlines } from './BlocksOrInlines';
import { ImageObjectOrString } from './ImageObjectOrString';
import { Integer } from './Integer';
import { Node } from './Node';
import { PropertyValueOrString } from './PropertyValueOrString';

// A single item in a list.
export class ListItem {
  type = "ListItem";

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
  name?: string;

  // The URL of the item.
  url?: string;

  // The content of the list item.
  content?: BlocksOrInlines;

  // The item represented by this list item.
  item?: Node;

  // A flag to indicate if this list item is checked.
  isChecked?: boolean;

  // The position of the item in a series or sequence of items.
  position?: Integer;

  constructor(options?: ListItem) {
    if (options) Object.assign(this, options)
    
  }
}
