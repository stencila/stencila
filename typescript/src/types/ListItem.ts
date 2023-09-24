// Generated file; do not edit. See `../rust/schema-gen` crate.

import { BlocksOrInlines } from './BlocksOrInlines';
import { Integer } from './Integer';
import { Node } from './Node';
import { Thing } from './Thing';

// A single item in a list.
export class ListItem extends Thing {
  type = "ListItem";

  // The content of the list item.
  content?: BlocksOrInlines;

  // The item represented by this list item.
  item?: Node;

  // A flag to indicate if this list item is checked.
  isChecked?: boolean;

  // The position of the item in a series or sequence of items.
  position?: Integer;

  constructor(options?: ListItem) {
    super()
    if (options) Object.assign(this, options)
    
  }

  static from(other: ListItem): ListItem {
    return new ListItem(other)
  }
}
