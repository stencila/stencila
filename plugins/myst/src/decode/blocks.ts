import { Block, CodeBlock, Heading, Paragraph } from "@stencila/types";
import type { FlowContent, Block as MySTBlock } from "myst-spec";

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
export function mdsToBlocks(mds: (MySTBlock | FlowContent)[]): Block[] {
  return mds.map((md) => {
    switch (md.type) {
      case "block":
        // TODO: do we need to support multiple MySTBlock?
        // Currently assuming only one at top level, see index.ts
        throw new Error(`Not yet implemented: ${md.type}`);
      case "mystDirective":
        // Technically Directive should not exist after basicTransformations() in index.ts
        throw new Error(`Not yet implemented: ${md.type}`);
      case "paragraph":
        return new Paragraph(mdsToInlines(md.children));
      case "heading":
        return new Heading(md.depth, mdsToInlines(md.children));
      case "code":
        return new CodeBlock(md.value);
      case "admonition":
      case "blockquote":
      case "container":
      case "definition":
      case "footnoteDefinition":
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
