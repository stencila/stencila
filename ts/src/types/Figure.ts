// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { CreativeWork } from "./CreativeWork.js";

/**
 * Encapsulates one or more images, videos, tables, etc, and provides captions and labels for them.
 */
export class Figure extends CreativeWork {
  type = "Figure";

  /**
   * The content of the figure.
   */
  content: Block[];

  /**
   * A short label for the figure.
   */
  label?: string;

  /**
   * A caption for the figure.
   */
  caption?: Block[];

  constructor(content: Block[], options?: Partial<Figure>) {
    super();
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Figure`
*/
export function figure(content: Block[], options?: Partial<Figure>): Figure {
  return new Figure(content, options);
}
