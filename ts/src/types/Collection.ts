// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CreativeWork } from "./CreativeWork.js";
import { CreativeWorkType } from "./CreativeWorkType.js";

/**
 * A collection of CreativeWorks or other artifacts.
 */
export class Collection extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Collection";

  /**
   * Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
   */
  parts: CreativeWorkType[];

  constructor(parts: CreativeWorkType[], options?: Partial<Collection>) {
    super();
    this.type = "Collection";
    if (options) Object.assign(this, options);
    this.parts = parts;
  }
}

/**
* Create a new `Collection`
*/
export function collection(parts: CreativeWorkType[], options?: Partial<Collection>): Collection {
  return new Collection(parts, options);
}
