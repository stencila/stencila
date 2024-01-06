// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Author } from "./Author.js";
import { Entity } from "./Entity.js";
import { ListItem } from "./ListItem.js";
import { ListOrder } from "./ListOrder.js";

/**
 * A list of items.
 */
export class List extends Entity {
  type = "List";

  /**
   * The items in the list.
   */
  items: ListItem[];

  /**
   * The ordering of the list.
   */
  order: ListOrder;

  /**
   * The authors of the list.
   */
  authors?: Author[];

  constructor(items: ListItem[], order: ListOrder, options?: Partial<List>) {
    super();
    if (options) Object.assign(this, options);
    this.items = items;
    this.order = order;
  }
}

/**
* Create a new `List`
*/
export function list(items: ListItem[], order: ListOrder, options?: Partial<List>): List {
  return new List(items, order, options);
}
