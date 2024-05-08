import { Block, Paragraph } from "@stencila/types";
import type { BlockContent } from "mdast";

import { mdsToInlines } from "./inlines.js";

/**
 * Transform MDAST `BlockContent` nodes to Stencila Schema `Block` nodes
 *
 * This is equivalent to the Rust `mds_to_blocks` function in
 * `rust/codec-markdown/src/decode/blocks.rs`.
 *
 * This is also an update of code in
 * https://github.com/stencila/encoda/blob/master/src/codecs/md/index.ts.
 */
export function mdsToBlocks(mds: BlockContent[]): Block[] {
  return mds.map((block) => {
    switch (block.type) {
      case "paragraph":
        return new Paragraph(mdsToInlines(block.children));
      case "heading":
      case "thematicBreak":
      case "blockquote":
      case "list":
      case "table":
      case "html":
      case "code":
        throw new Error(`Not yet implemented: ${block.type}`);
    }
  });
}
