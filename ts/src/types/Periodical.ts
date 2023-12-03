// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CreativeWork } from "./CreativeWork.js";
import { Date } from "./Date.js";

/**
 * A periodical publication.
 */
export class Periodical extends CreativeWork {
  type = "Periodical";

  /**
   * The date this Periodical was first published.
   */
  dateStart?: Date;

  /**
   * The date this Periodical ceased publication.
   */
  dateEnd?: Date;

  /**
   * The International Standard Serial Number(s) (ISSN) that identifies this serial publication.
   */
  issns?: string[];

  constructor(options?: Partial<Periodical>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Periodical`
*/
export function periodical(options?: Partial<Periodical>): Periodical {
  return new Periodical(options);
}
