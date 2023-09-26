// Generated file; do not edit. See `../rust/schema-gen` crate.

import { PersonOrOrganization } from "./PersonOrOrganization.js";
import { Thing } from "./Thing.js";

/**
 * A grant, typically financial or otherwise quantifiable, of resources.
 */
export class Grant extends Thing {
  type = "Grant";

  /**
   * Indicates an item funded or sponsored through a Grant.
   */
  fundedItems?: Thing[];

  /**
   * A person or organization that supports a thing through a pledge, promise, or financial contribution.
   */
  sponsors?: PersonOrOrganization[];

  constructor(options?: Partial<Grant>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Grant`
*/
export function grant(options?: Partial<Grant>): Grant {
  return new Grant(options);
}
