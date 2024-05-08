import { paragraph } from "@stencila/types";
import { mdsToInlines } from "./inlines";
/**
 * Transform MDAST `BlockContent` nodes to Stencila Schema `Block` nodes
 *
 * This is equivalent to the Rust `mds_to_blocks` function in
 * `rust/codec-markdown/src/decode/blocks.rs`.
 *
 * It is a also an update of the TypeScript function
 * TODO in `TODO`.
 */
export function mdsToBlocks(mds) {
    return mds.map((block) => {
        switch (block.type) {
            case "paragraph":
                return paragraph(mdsToInlines(block.children));
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
