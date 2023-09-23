// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CreativeWork } from './CreativeWork';
import { CreativeWorkType } from './CreativeWorkType';

// A collection of CreativeWorks or other artifacts.
export class Collection extends CreativeWork {
  type = "Collection";

  constructor(parts: CreativeWorkType[], options?: Collection) {
    super()
    if (options) Object.assign(this, options)
    this.parts = parts;
  }
}
