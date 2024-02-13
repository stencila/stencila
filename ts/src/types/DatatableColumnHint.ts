// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Integer } from "./Integer.js";
import { Primitive } from "./Primitive.js";

/**
 * A hint to the type and values in a `DatatableColumn`.
 */
export class DatatableColumnHint extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "DatatableColumnHint";

  /**
   * The name of the column.
   */
  name: string;

  /**
   * The type of items in the column.
   */
  itemType: string;

  /**
   * The minimum value in the column.
   */
  minimum?: Primitive;

  /**
   * The maximum value in the column.
   */
  maximum?: Primitive;

  /**
   * The number of `Null` values in the column.
   */
  nulls?: Integer;

  constructor(name: string, itemType: string, options?: Partial<DatatableColumnHint>) {
    super();
    this.type = "DatatableColumnHint";
    if (options) Object.assign(this, options);
    this.name = name;
    this.itemType = itemType;
  }
}

/**
* Create a new `DatatableColumnHint`
*/
export function datatableColumnHint(name: string, itemType: string, options?: Partial<DatatableColumnHint>): DatatableColumnHint {
  return new DatatableColumnHint(name, itemType, options);
}
