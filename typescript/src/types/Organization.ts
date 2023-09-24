// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Brand } from './Brand';
import { ContactPoint } from './ContactPoint';
import { ImageObjectOrString } from './ImageObjectOrString';
import { OrganizationOrPerson } from './OrganizationOrPerson';
import { PostalAddressOrString } from './PostalAddressOrString';
import { Thing } from './Thing';

// An organization such as a school, NGO, corporation, club, etc.
export class Organization extends Thing {
  type = "Organization";

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
  legalName?: string;

  // The logo of the organization.
  logo?: ImageObjectOrString;

  // Person(s) or organization(s) who are members of this organization.
  members?: OrganizationOrPerson[];

  // Entity that the Organization is a part of. For example, parentOrganization to a department is a university.
  parentOrganization?: Organization;

  constructor(options?: Organization) {
    super()
    if (options) Object.assign(this, options)
    
  }

  static from(other: Organization): Organization {
    return new Organization(other)
  }
}
