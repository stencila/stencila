// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { PropertyValueOrString } from './PropertyValueOrString';
import { String } from './String';

// A contact point, usually within an organization.
export class ContactPoint {
  // The type of this item
  type = "ContactPoint";

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
  name?: String;

  // The URL of the item.
  url?: String;

  // Email address for correspondence.
  emails?: String[];

  // Telephone numbers for the contact point.
  telephoneNumbers?: String[];

  // Languages (human not programming) in which it is possible to communicate
  // with the organization/department etc.
  availableLanguages?: String[];

  constructor(options?: ContactPoint) {
    if (options) Object.assign(this, options)
    
  }
}
