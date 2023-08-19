// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { PropertyValueOrString } from './PropertyValueOrString';
import { String } from './String';

// A word, name, acronym, phrase, etc. with a formal definition.
export class DefinedTerm {
  // The type of this item
  type = "DefinedTerm";

  // The identifier for this item
  id?: String;

  // Alternate names (aliases) for the item.
  alternateNames?: String[];

  // A description of the item.
  description?: Block[];

  // Any kind of identifier for any kind of Thing.
  identifiers?: PropertyValueOrString[];

  // Images of the item.
  images?: ImageObjectOrString[];

  // The name of the item.
  name: String;

  // The URL of the item.
  url?: String;

  // A code that identifies this DefinedTerm within a DefinedTermSet
  termCode?: String;

  constructor(name: String, options?: DefinedTerm) {
    if (options) Object.assign(this, options)
    this.name = name;
  }
}
