// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from './Entity';
import { ListItem } from './ListItem';
import { ListOrder } from './ListOrder';

// A list of items.
export class List extends Entity {
  type = "List";

  // The items in the list.
  items: ListItem[];

  // The ordering of the list.
  order: ListOrder;

  constructor(items: ListItem[], order: ListOrder, options?: List) {
    super()
    if (options) Object.assign(this, options)
    this.items = items;
    this.order = order;
  }

  static from(other: List): List {
    return new List(other.items!, other.order!, other)
  }
}
