// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CreativeWork } from "./CreativeWork.js";

/**
 * A software application.
 */
export class SoftwareApplication extends CreativeWork {
  type = "SoftwareApplication";

  /**
   * The name of the item.
   */
  name: string;

  /**
   * Requirements for application, including shared libraries that are not included in the application distribution.
   */
  softwareRequirements?: SoftwareApplication[];

  /**
   * Version of the software.
   */
  softwareVersion?: string;

  constructor(name: string, options?: Partial<SoftwareApplication>) {
    super();
    if (options) Object.assign(this, options);
    this.name = name;
  }
}

/**
* Create a new `SoftwareApplication`
*/
export function softwareApplication(name: string, options?: Partial<SoftwareApplication>): SoftwareApplication {
  return new SoftwareApplication(name, options);
}
