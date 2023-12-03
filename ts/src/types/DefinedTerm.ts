// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Thing } from "./Thing.js";

/**
 * A word, name, acronym, phrase, etc. with a formal definition.
 */
export class DefinedTerm extends Thing {
  type = "DefinedTerm";

  /**
   * The name of the item.
   */
  name: string;

  /**
   * A code that identifies this DefinedTerm within a DefinedTermSet
   */
  termCode?: string;

  constructor(name: string, options?: Partial<DefinedTerm>) {
    super();
    if (options) Object.assign(this, options);
    this.name = name;
  }
}

/**
* Create a new `DefinedTerm`
*/
export function definedTerm(name: string, options?: Partial<DefinedTerm>): DefinedTerm {
  return new DefinedTerm(name, options);
}
