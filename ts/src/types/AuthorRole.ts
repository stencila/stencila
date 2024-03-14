// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { AuthorRoleName } from "./AuthorRoleName.js";
import { PersonOrOrganizationOrSoftwareApplication } from "./PersonOrOrganizationOrSoftwareApplication.js";
import { Role } from "./Role.js";
import { Timestamp } from "./Timestamp.js";

/**
 * An author and their role.
 */
export class AuthorRole extends Role {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "AuthorRole";

  /**
   * The author.
   */
  author: PersonOrOrganizationOrSoftwareApplication;

  /**
   * A role played by the author.
   */
  roleName: AuthorRoleName;

  /**
   * Timestamp of most recent modification by the author in the role.
   */
  lastModified?: Timestamp;

  constructor(author: PersonOrOrganizationOrSoftwareApplication, roleName: AuthorRoleName, options?: Partial<AuthorRole>) {
    super();
    this.type = "AuthorRole";
    if (options) Object.assign(this, options);
    this.author = author;
    this.roleName = roleName;
  }
}

/**
* Create a new `AuthorRole`
*/
export function authorRole(author: PersonOrOrganizationOrSoftwareApplication, roleName: AuthorRoleName, options?: Partial<AuthorRole>): AuthorRole {
  return new AuthorRole(author, roleName, options);
}
