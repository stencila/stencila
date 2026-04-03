// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CreativeWork } from "./CreativeWork.js";

/**
 * Encapsulates one or more images, videos, tables, etc, and provides captions and labels for them.
 */
export class Figure extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Figure";

  /**
   * A short label for the figure.
   */
  label?: string;

  /**
   * Whether the label should be automatically updated.
   */
  labelAutomatically?: boolean;

  /**
   * A caption for the figure.
   */
  caption?: Block[];

  /**
   * Layout for arranging content blocks in a multi-panel figure. When absent, content blocks stack vertically with no grid.
   */
  layout?: string;

  /**
   * An optional SVG overlay rendered on top of the figure's content. The SVG is positioned absolutely over the content area and scales proportionally using the SVG viewBox. Used for annotations such as arrows, callouts, bounding boxes, and labels.
   */
  overlay?: string;

  /**
   * Padding around the figure's content area in pixel units. Creates whitespace where overlay annotations can be placed outside the image bounds. Accepts 1, 2, or 4 space-separated values following CSS shorthand order (all, vertical/horizontal, or top/right/bottom/left).
   */
  padding?: string;

  /**
   * The content of the figure.
   */
  content: Block[];

  constructor(content: Block[], options?: Partial<Figure>) {
    super();
    this.type = "Figure";
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
