// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Block } from './Block';
import { BlocksOrInlines } from './BlocksOrInlines';
import { Boolean } from './Boolean';
import { ImageObjectOrString } from './ImageObjectOrString';
import { Integer } from './Integer';
import { Node } from './Node';
import { PropertyValueOrString } from './PropertyValueOrString';
import { String } from './String';

// A single item in a list.
export class ListItem {
  // The type of this item
  type = "ListItem";

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
  name?: String;

  // The URL of the item.
  url?: String;

  // The content of the list item.
  content?: BlocksOrInlines;

  // The item represented by this list item.
  item?: Node;

  // A flag to indicate if this list item is checked.
  isChecked?: Boolean;

  // The position of the item in a series or sequence of items.
  position?: Integer;

  constructor(options?: ListItem) {
    if (options) Object.assign(this, options)
    
  }
}
