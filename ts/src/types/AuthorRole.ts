// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { AuthorRoleAuthor } from "./AuthorRoleAuthor.js";
import { AuthorRoleName } from "./AuthorRoleName.js";
import { Role } from "./Role.js";
import { Timestamp } from "./Timestamp.js";

/**
 * An author and their role.
 */
export class AuthorRole extends Role {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "AuthorRole";

  /**
   * The entity acting as an author.
   */
  author: AuthorRoleAuthor;

  /**
   * The role played by the author.
   */
  roleName: AuthorRoleName;

  /**
   * The format that the author used to perform the role. e.g. Markdown, Python
   */
  format?: string;

  /**
   * Timestamp of most recent modification, by the author, in the role.
   */
  lastModified?: Timestamp;

  constructor(author: AuthorRoleAuthor, roleName: AuthorRoleName, options?: Partial<AuthorRole>) {
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
export function authorRole(author: AuthorRoleAuthor, roleName: AuthorRoleName, options?: Partial<AuthorRole>): AuthorRole {
  return new AuthorRole(author, roleName, options);
}
