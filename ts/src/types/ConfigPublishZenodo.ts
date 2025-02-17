// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { ConfigPublishZenodoAccessRight } from "./ConfigPublishZenodoAccessRight.js";
import { Date } from "./Date.js";

/**
 * Zenodo publishing options.
 */
export class ConfigPublishZenodo {
  /**
   * The date of embargoed.
   */
  embargoed?: Date;

  /**
   * The access right of the document.
   */
  accessRight?: ConfigPublishZenodoAccessRight;

  /**
   * extra notes about deposition.
   */
  notes?: string;

  /**
   * The methodology of the study.
   */
  method?: string;

  constructor(options?: Partial<ConfigPublishZenodo>) {
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `ConfigPublishZenodo`
*/
export function configPublishZenodo(options?: Partial<ConfigPublishZenodo>): ConfigPublishZenodo {
  return new ConfigPublishZenodo(options);
}
