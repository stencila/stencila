// Generated file. Do not edit; see `rust/schema-gen` crate.

import { ListItem } from './ListItem';
import { ListOrder } from './ListOrder';
import { String } from './String';

// A list of items.
export class List {
  // The type of this item
  type = "List";

  // The identifier for this item
  id?: String;

  // The items in the list.
  items: ListItem[];

  // The ordering of the list.
  order: ListOrder;

  constructor(items: ListItem[], order: ListOrder, options?: List) {
    if (options) Object.assign(this, options)
    this.items = items;
    this.order = order;
  }
}
