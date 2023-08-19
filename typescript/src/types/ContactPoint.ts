// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { PropertyValueOrString } from './PropertyValueOrString';

// A contact point, usually within an organization.
export class ContactPoint {
  type = "ContactPoint";

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
  name?: string;

  // The URL of the item.
  url?: string;

  // Email address for correspondence.
  emails?: string[];

  // Telephone numbers for the contact point.
  telephoneNumbers?: string[];

  // Languages (human not programming) in which it is possible to communicate
  // with the organization/department etc.
  availableLanguages?: string[];

  constructor(options?: ContactPoint) {
    if (options) Object.assign(this, options)
    
  }
}
