// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CodeExecutable } from "./CodeExecutable.js";
import { CompilationDigest } from "./CompilationDigest.js";
import { CompilationMessage } from "./CompilationMessage.js";
import { Cord } from "./Cord.js";
import { LabelType } from "./LabelType.js";
import { Node } from "./Node.js";

/**
 * An executable code chunk.
 */
export class CodeChunk extends CodeExecutable {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CodeChunk";

  /**
   * The type of the label for the chunk.
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
   * A caption for the chunk.
   */
  caption?: Block[];

  /**
   * An optional SVG overlay rendered on top of the chunk's visual content. The SVG is positioned absolutely over the content area and scales proportionally using the SVG viewBox. Used for annotations such as arrows, callouts, bounding boxes, and labels.
   */
  overlay?: string;

  /**
   * The compiled SVG overlay with all custom elements expanded to standard SVG. Generated during compilation from the overlay source. When present, renderers use this instead of overlay.
   */
  overlayCompiled?: string;

  /**
   * A digest of the overlay property.
   */
  overlayCompilationDigest?: CompilationDigest;

  /**
   * Messages generated while compiling the overlay.
   */
  overlayCompilationMessages?: CompilationMessage[];

  /**
   * Outputs from executing the chunk.
   */
  outputs?: Node[];

  /**
   * Whether the code should be displayed to the reader.
   */
  isEchoed?: boolean;

  /**
   * Whether the outputs should be hidden from the reader.
   */
  isHidden?: boolean;

  /**
   * Whether the code should be treated as side-effect free when executed.
   */
  executionPure?: boolean;

  constructor(code: Cord, options?: Partial<CodeChunk>) {
    super(code);
    this.type = "CodeChunk";
    if (options) Object.assign(this, options);
    this.code = code;
  }
}

/**
* Create a new `CodeChunk`
*/
export function codeChunk(code: Cord, options?: Partial<CodeChunk>): CodeChunk {
  return new CodeChunk(code, options);
}
