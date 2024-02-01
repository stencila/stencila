// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Thing } from "./Thing.js";

/**
 * A contact point, usually within an organization.
 */
export class ContactPoint extends Thing {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ContactPoint";

  /**
   * Email address for correspondence.
   */
  emails?: string[];

  /**
   * Telephone numbers for the contact point.
   */
  telephoneNumbers?: string[];

  /**
   * Languages (human not programming) in which it is possible to communicate with the organization/department etc.
   */
  availableLanguages?: string[];

  constructor(options?: Partial<ContactPoint>) {
    super();
    this.type = "ContactPoint";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `ContactPoint`
*/
export function contactPoint(options?: Partial<ContactPoint>): ContactPoint {
  return new ContactPoint(options);
}
