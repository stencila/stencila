// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { Entity } from "./Entity.js";
import { ListItem } from "./ListItem.js";
import { ListOrder } from "./ListOrder.js";

/**
 * A list of items.
 */
export class List extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "List";

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
    this.type = "List";
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
