// Generated file; do not edit. See `../rust/schema-gen` crate.

import { ListItem } from './ListItem';
import { ListOrder } from './ListOrder';

// A list of items.
export class List {
  type = "List";

  // The identifier for this item
  id?: string;

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
