// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { PropertyValueOrString } from './PropertyValueOrString';
import { String } from './String';

// A physical mailing address.
export class PostalAddress {
  // The type of this item
  type = "PostalAddress";

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

  // The street address.
  streetAddress?: String;

  // The post office box number.
  postOfficeBoxNumber?: String;

  // The locality in which the street address is, and which is in the region.
  addressLocality?: String;

  // The region in which the locality is, and which is in the country.
  addressRegion?: String;

  // The postal code.
  postalCode?: String;

  // The country.
  addressCountry?: String;

  constructor(options?: PostalAddress) {
    if (options) Object.assign(this, options)
    
  }
}
