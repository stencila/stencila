// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Thing } from './Thing';

// A contact point, usually within an organization.
export class ContactPoint extends Thing {
  type = "ContactPoint";

  // Email address for correspondence.
  emails?: string[];

  // Telephone numbers for the contact point.
  telephoneNumbers?: string[];

  // Languages (human not programming) in which it is possible to communicate
  // with the organization/department etc.
  availableLanguages?: string[];

  constructor(options?: ContactPoint) {
    super()
    if (options) Object.assign(this, options)
    
  }
}
