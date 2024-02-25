// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CreativeWork } from "./CreativeWork.js";

/**
 * A software application.
 */
export class SoftwareApplication extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "SoftwareApplication";

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

  /**
   * Operating systems supported (e.g. Windows 7, OS X 10.6).
   */
  operatingSystem?: string;

  constructor(name: string, options?: Partial<SoftwareApplication>) {
    super();
    this.type = "SoftwareApplication";
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
