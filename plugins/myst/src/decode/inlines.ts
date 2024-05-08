import {
  Emphasis,
  Inline,
  Strong,
  Text,
} from "@stencila/types";
import type { PhrasingContent } from "mdast";

/**
 * Transform MDAST `PhrasingContent` to Stencila Schema `Inline` nodes
 *
 * This is equivalent to the Rust `mds_to_inlines` function in
 * `rust/codec-markdown/src/decode/inlines.rs`.
 *
 * This is also an update of code in
 * https://github.com/stencila/encoda/blob/master/src/codecs/md/index.ts.
 */
export function mdsToInlines(mds: PhrasingContent[]): Inline[] {
  return mds.map((inline) => {
    switch (inline.type) {
      case "text":
        return new Text(inline.value);
      case "emphasis":
        return new Emphasis(mdsToInlines(inline.children));
      case "strong":
        return new Strong(mdsToInlines(inline.children));
      case "link":
      case "linkReference":
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
