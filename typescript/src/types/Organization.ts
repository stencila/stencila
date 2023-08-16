// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Block } from './Block';
import { Brand } from './Brand';
import { ContactPoint } from './ContactPoint';
import { ImageObjectOrString } from './ImageObjectOrString';
import { OrganizationOrPerson } from './OrganizationOrPerson';
import { PostalAddressOrString } from './PostalAddressOrString';
import { PropertyValueOrString } from './PropertyValueOrString';
import { String } from './String';

// An organization such as a school, NGO, corporation, club, etc.
export class Organization {
  // The type of this item
  type = "Organization";

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

  // Postal address for the organization.
  address?: PostalAddressOrString;

  // Brands that the organization is connected with.
  brands?: Brand[];

  // Correspondence/Contact points for the organization.
  contactPoints?: ContactPoint[];

  // Departments within the organization. For example, Department of Computer Science, Research & Development etc.
  departments?: Organization[];

  // Organization(s) or person(s) funding the organization.
  funders?: OrganizationOrPerson[];

  // The official name of the organization, e.g. the registered company name.
  legalName?: String;

  // The logo of the organization.
  logo?: ImageObjectOrString;

  // Person(s) or organization(s) who are members of this organization.
  members?: OrganizationOrPerson[];

  // Entity that the Organization is a part of. For example, parentOrganization to a department is a university.
  parentOrganization?: Organization;

  constructor(options?: Organization) {
    if (options) Object.assign(this, options)
    
  }
}
