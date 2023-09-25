// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Brand } from "./Brand.js";
import { ContactPoint } from "./ContactPoint.js";
import { ImageObjectOrString } from "./ImageObjectOrString.js";
import { OrganizationOrPerson } from "./OrganizationOrPerson.js";
import { PostalAddressOrString } from "./PostalAddressOrString.js";
import { Thing } from "./Thing.js";

/**
 * An organization such as a school, NGO, corporation, club, etc.
 */
export class Organization extends Thing {
  type = "Organization";

  /**
   * Postal address for the organization.
   */
  address?: PostalAddressOrString;

  /**
   * Brands that the organization is connected with.
   */
  brands?: Brand[];

  /**
   * Correspondence/Contact points for the organization.
   */
  contactPoints?: ContactPoint[];

  /**
   * Departments within the organization. For example, Department of Computer Science, Research & Development etc.
   */
  departments?: Organization[];

  /**
   * Organization(s) or person(s) funding the organization.
   */
  funders?: OrganizationOrPerson[];

  /**
   * The official name of the organization, e.g. the registered company name.
   */
  legalName?: string;

  /**
   * The logo of the organization.
   */
  logo?: ImageObjectOrString;

  /**
   * Person(s) or organization(s) who are members of this organization.
   */
  members?: OrganizationOrPerson[];

  /**
   * Entity that the Organization is a part of. For example, parentOrganization to a department is a university.
   */
  parentOrganization?: Organization;

  constructor(options?: Partial<Organization>) {
    super();
    if (options) Object.assign(this, options);
    
  }

  /**
  * Create a `Organization` from an object
  */
  static from(other: Organization): Organization {
    return new Organization(other);
  }
}
