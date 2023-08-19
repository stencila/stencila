// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { PropertyValueOrString } from './PropertyValueOrString';

// A word, name, acronym, phrase, etc. with a formal definition.
export class DefinedTerm {
  type = "DefinedTerm";

  // The identifier for this item
  id?: string;

  // Alternate names (aliases) for the item.
  alternateNames?: string[];

  // A description of the item.
  description?: Block[];

  // Any kind of identifier for any kind of Thing.
  identifiers?: PropertyValueOrString[];

  // Images of the item.
  images?: ImageObjectOrString[];

  // The name of the item.
  name: string;

  // The URL of the item.
  url?: string;

  // A code that identifies this DefinedTerm within a DefinedTermSet
  termCode?: string;

  constructor(name: string, options?: DefinedTerm) {
    if (options) Object.assign(this, options)
    this.name = name;
  }
}
