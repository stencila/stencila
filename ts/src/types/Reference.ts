// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { CreativeWorkType } from "./CreativeWorkType.js";
import { Date } from "./Date.js";
import { Entity } from "./Entity.js";
import { IntegerOrString } from "./IntegerOrString.js";

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
   * The authors of the work.
   */
  authors?: Author[];

  /**
   * Date of first publication.
   */
  date?: Date;

  /**
   * The title of the work.
   */
  title?: string;

  /**
   * An other `CreativeWork` that the reference is a part of.
   */
  isPartOf?: CreativeWorkType;

  /**
   * The page on which the article starts; for example "135" or "xiii".
   */
  pageStart?: IntegerOrString;

  /**
   * The page on which the article ends; for example "138" or "xvi".
   */
  pageEnd?: IntegerOrString;

  /**
   * Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".
   */
  pagination?: string;

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
