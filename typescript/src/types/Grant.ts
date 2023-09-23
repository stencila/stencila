// Generated file; do not edit. See `../rust/schema-gen` crate.

import { PersonOrOrganization } from './PersonOrOrganization';
import { Thing } from './Thing';

// A grant, typically financial or otherwise quantifiable, of resources.
export class Grant extends Thing {
  type = "Grant";

  // Indicates an item funded or sponsored through a Grant.
  fundedItems?: Thing[];

  // A person or organization that supports a thing through a pledge, promise, or financial contribution.
  sponsors?: PersonOrOrganization[];

  constructor(options?: Grant) {
    super()
    if (options) Object.assign(this, options)
    
  }
}
