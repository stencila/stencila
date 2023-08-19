// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { Organization } from './Organization';
import { OrganizationOrPerson } from './OrganizationOrPerson';
import { PostalAddressOrString } from './PostalAddressOrString';
import { PropertyValueOrString } from './PropertyValueOrString';

// A person (alive, dead, undead, or fictional).
export class Person {
  type = "Person";

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

  // Postal address for the person.
  address?: PostalAddressOrString;

  // Organizations that the person is affiliated with.
  affiliations?: Organization[];

  // Email addresses for the person.
  emails?: string[];

  // Family name. In the U.S., the last name of a person.
  familyNames?: string[];

  // A person or organization that supports (sponsors) something through
  // some kind of financial contribution.
  funders?: OrganizationOrPerson[];

  // Given name. In the U.S., the first name of a person.
  givenNames?: string[];

  // An honorific prefix preceding a person's name such as Dr/Mrs/Mr.
  honorificPrefix?: string;

  // An honorific suffix after a person's name such as MD/PhD/MSCSW.
  honorificSuffix?: string;

  // The job title of the person (for example, Financial Manager).
  jobTitle?: string;

  // An organization (or program membership) to which this person belongs.
  memberOf?: Organization[];

  // Telephone numbers for the person.
  telephoneNumbers?: string[];

  constructor(options?: Person) {
    if (options) Object.assign(this, options)
    
  }
}
