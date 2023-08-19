// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { PropertyValueOrString } from './PropertyValueOrString';

// A physical mailing address.
export class PostalAddress {
  type = "PostalAddress";

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

  // The street address.
  streetAddress?: string;

  // The post office box number.
  postOfficeBoxNumber?: string;

  // The locality in which the street address is, and which is in the region.
  addressLocality?: string;

  // The region in which the locality is, and which is in the country.
  addressRegion?: string;

  // The postal code.
  postalCode?: string;

  // The country.
  addressCountry?: string;

  constructor(options?: PostalAddress) {
    if (options) Object.assign(this, options)
    
  }
}
