// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Thing } from './Thing';

// A word, name, acronym, phrase, etc. with a formal definition.
export class DefinedTerm extends Thing {
  type = "DefinedTerm";

  // A code that identifies this DefinedTerm within a DefinedTermSet
  termCode?: string;

  constructor(name: string, options?: DefinedTerm) {
    super()
    if (options) Object.assign(this, options)
    this.name = name;
  }

  static from(other: DefinedTerm): DefinedTerm {
    return new DefinedTerm(other.name!, other)
  }
}
