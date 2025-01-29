// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { ConfigPublishGhost } from "./ConfigPublishGhost.js";
import { ConfigPublishZenodo } from "./ConfigPublishZenodo.js";

/**
 * Publishing options.
 */
export class ConfigPublish {
  /**
   * Ghost publishing options.
   */
  ghost?: ConfigPublishGhost;

  /**
   * Zenodo publishing options.
   */
  zenodo?: ConfigPublishZenodo;

  constructor(options?: Partial<ConfigPublish>) {
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `ConfigPublish`
*/
export function configPublish(options?: Partial<ConfigPublish>): ConfigPublish {
  return new ConfigPublish(options);
}
