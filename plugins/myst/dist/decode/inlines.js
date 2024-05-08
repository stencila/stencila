import { Emphasis, Text } from "@stencila/types";
/**
 * Transform MDAST `PhrasingContent` to Stencila Schema `Inline` nodes
 *
 * This is equivalent to the Rust `mds_to_inlines` function in
 * `rust/codec-markdown/src/decode/inlines.rs`.
 *
 * It is a also an update of the TypeScript function
 * TODO in `TODO`.
 */
export function mdsToInlines(mds) {
    return mds.map((inline) => {
        switch (inline.type) {
            case "text":
                return new Text(inline.value);
            case "emphasis":
                return new Emphasis(mdsToInlines(inline.children));
            case "link":
            case "linkReference":
            case "strong":
            case "delete":
            case "html":
            case "inlineCode":
            case "break":
            case "image":
            case "imageReference":
            case "footnote":
            case "footnoteReference":
                throw new Error(`Not yet implemented: ${inline.type}`);
        }
    });
}
