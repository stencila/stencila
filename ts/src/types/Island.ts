// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Entity } from "./Entity.js";
import { LabelType } from "./LabelType.js";

/**
 * An island of content in a document.
 */
export class Island extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Island";

  /**
   * The content within the section.
   */
  content: Block[];

  /**
   * Whether the island is automatically generated.
   */
  isAutomatic?: boolean;

  /**
   * The type of the label for the island.
   */
  labelType?: LabelType;

  /**
   * A short label for the chunk.
   */
  label?: string;

  /**
   * Whether the label should be automatically updated.
   */
  labelAutomatically?: boolean;

  /**
   * Other IDs for the island, in addition to the primary `id`.
   */
  otherIds?: string[];

  /**
   * The style to apply to the island.
   */
  style?: string;

  constructor(content: Block[], options?: Partial<Island>) {
    super();
    this.type = "Island";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Island`
*/
export function island(content: Block[], options?: Partial<Island>): Island {
  return new Island(content, options);
}
