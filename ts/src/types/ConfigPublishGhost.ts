// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { ConfigPublishGhostState } from "./ConfigPublishGhostState.js";
import { Date } from "./Date.js";

/**
 * Ghost publishing options.
 */
export class ConfigPublishGhost {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ConfigPublishGhost";

  /**
   * The URL slug for the page or post.
   */
  slug?: string;

  /**
   * Whether the page or post is featured.
   */
  featured?: boolean;

  /**
   * The date that the page or post is to be published.
   */
  schedule?: Date;

  /**
   * the state of the page or post eg draft or published.
   */
  state?: ConfigPublishGhostState;

  /**
   * ghost tags.
   */
  tags?: string[];

  constructor(options?: Partial<ConfigPublishGhost>) {
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `ConfigPublishGhost`
*/
export function configPublishGhost(options?: Partial<ConfigPublishGhost>): ConfigPublishGhost {
  return new ConfigPublishGhost(options);
}
