// Generated file; do not edit. See `../rust/schema-gen` crate.

import { AuthorRoleName } from "./AuthorRoleName.js";
import { PersonOrOrganizationOrSoftwareApplication } from "./PersonOrOrganizationOrSoftwareApplication.js";
import { Role } from "./Role.js";

/**
 * An author and their role.
 */
export class AuthorRole extends Role {
  type = "AuthorRole";

  /**
   * The author.
   */
  author: PersonOrOrganizationOrSoftwareApplication;

  /**
   * A role played by the author.
   */
  roleName: AuthorRoleName;

  constructor(author: PersonOrOrganizationOrSoftwareApplication, roleName: AuthorRoleName, options?: Partial<AuthorRole>) {
    super();
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
