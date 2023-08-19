// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { Organization } from './Organization';
import { OrganizationOrPerson } from './OrganizationOrPerson';
import { PostalAddressOrString } from './PostalAddressOrString';
import { PropertyValueOrString } from './PropertyValueOrString';
import { String } from './String';

// A person (alive, dead, undead, or fictional).
export class Person {
  // The type of this item
  type = "Person";

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

  // Postal address for the person.
  address?: PostalAddressOrString;

  // Organizations that the person is affiliated with.
  affiliations?: Organization[];

  // Email addresses for the person.
  emails?: String[];

  // Family name. In the U.S., the last name of a person.
  familyNames?: String[];

  // A person or organization that supports (sponsors) something through
  // some kind of financial contribution.
  funders?: OrganizationOrPerson[];

  // Given name. In the U.S., the first name of a person.
  givenNames?: String[];

  // An honorific prefix preceding a person's name such as Dr/Mrs/Mr.
  honorificPrefix?: String;

  // An honorific suffix after a person's name such as MD/PhD/MSCSW.
  honorificSuffix?: String;

  // The job title of the person (for example, Financial Manager).
  jobTitle?: String;

  // An organization (or program membership) to which this person belongs.
  memberOf?: Organization[];

  // Telephone numbers for the person.
  telephoneNumbers?: String[];

  constructor(options?: Person) {
    if (options) Object.assign(this, options)
    
  }
}
