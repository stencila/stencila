import { Block, Paragraph } from "@stencila/types";
import type { Directive, FlowContent } from "myst-spec";

import { mdsToInlines } from "./inlines.js";

/**
 * Transform MyST `Block` nodes to Stencila Schema `Block` nodes
 *
 * This is equivalent to the Rust `mds_to_blocks` function in
 * `rust/codec-markdown/src/decode/blocks.rs`.
 *
 * This is also an update of code in
 * https://github.com/stencila/encoda/blob/master/src/codecs/md/index.ts.
 */
export function mdsToBlocks(mds: FlowContent[]): Block[] {
  return mds.map((md) => {
    switch (md.type) {
      case "paragraph":
        return new Paragraph(mdsToInlines(md.children));
      case "mystDirective":
        return directiveToBlock(md);
      case "admonition":
      case "blockquote":
      case "code":
      case "container":
      case "definition":
      case "footnoteDefinition":
      case "heading":
      case "html":
      case "list":
      case "math":
      case "mystComment":
      case "mystTarget":
      case "table":
      case "thematicBreak":
        throw new Error(`Not yet implemented: ${md.type}`);
    }
  });
}

/**
 * Transform a MyST `Directive` into a Stencila `Block` node
 */
function directiveToBlock(directive: Directive): Block {
  switch (directive.name) {
    default:
      throw new Error(`mystRole not yet implemented: ${directive.name}`);
  }
}
