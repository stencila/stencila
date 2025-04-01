// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Date } from "./Date.js";
import { Entity } from "./Entity.js";

/**
 * A reference to a creative work, including books, movies, photographs, software programs, etc.
 */
export class Reference extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Reference";

  /**
   * The Digital Object Identifier for the work.
   */
  doi?: string;

  /**
   * The title of the work.
   */
  title?: string;

  /**
   * Date of first publication.
   */
  date?: Date;

  /**
   * The authors of the work.
   */
  authors?: string[];

  constructor(options?: Partial<Reference>) {
    super();
    this.type = "Reference";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Reference`
*/
export function reference(options?: Partial<Reference>): Reference {
  return new Reference(options);
}
