// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Brand } from "./Brand.js";
import { ContactPoint } from "./ContactPoint.js";
import { ImageObject } from "./ImageObject.js";
import { PersonOrOrganization } from "./PersonOrOrganization.js";
import { PostalAddressOrString } from "./PostalAddressOrString.js";
import { Thing } from "./Thing.js";

/**
 * An organization such as a school, NGO, corporation, club, etc.
 */
export class Organization extends Thing {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Organization";

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
  funders?: PersonOrOrganization[];

  /**
   * The official name of the organization, e.g. the registered company name.
   */
  legalName?: string;

  /**
   * The logo of the organization.
   */
  logo?: ImageObject;

  /**
   * Person(s) or organization(s) who are members of this organization.
   */
  members?: PersonOrOrganization[];

  /**
   * Entity that the Organization is a part of. For example, parentOrganization to a department is a university.
   */
  parentOrganization?: Organization;

  constructor(options?: Partial<Organization>) {
    super();
    this.type = "Organization";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Organization`
*/
export function organization(options?: Partial<Organization>): Organization {
  return new Organization(options);
}
