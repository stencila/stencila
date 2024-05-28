import { Block } from "@stencila/types";
import { MySTEncodeContext } from "./encoder.js";
import { encodeInlines } from "./inlines.js";

/**
 * Encode an array of Stencila `Block` nodes to MyST
 */
export function encodeBlocks(blocks: Block[], context: MySTEncodeContext) {
  for (const block of blocks) {
    encodeBlock(block, context);
  }
}

/**
 * Encode a Stencila `Block` node to MyST
 */
export function encodeBlock(block: Block, context: MySTEncodeContext) {
  context.enterNode(block.type, block.id ?? "");

  switch (block.type) {
    case "Paragraph": {
      encodeInlines(block.content, context);
      context.pushString("\n\n");
    }
  }

  context.exitNode();
}
