import {
  CodeInline,
  Emphasis,
  Inline,
  Strong,
  Text,
  Underline,
} from "@stencila/types";
import type { PhrasingContent } from "myst-spec";

/**
 * Transform MyST `PhrasingContent` to Stencila Schema `Inline` nodes
 *
 * This is equivalent to the Rust `mds_to_inlines` function in
 * `rust/codec-markdown/src/decode/inlines.rs`.
 *
 * This is also an update of code in
 * https://github.com/stencila/encoda/blob/master/src/codecs/md/index.ts.
 */
export function mdsToInlines(mds: PhrasingContent[]): Inline[] {
  return mds.map((md) => {
    switch (md.type) {
      case "text":
        return new Text(md.value);
      case "emphasis":
        return new Emphasis(mdsToInlines(md.children));
      case "strong":
        return new Strong(mdsToInlines(md.children));
      case "inlineCode":
        return new CodeInline(md.value);
      case "mystRole":
        // Technically Roles should not exist after basicTransformations() in index.ts
        throw new Error(`mdast inline type not yet implemented: ${md.type}`);
      case "underline":
        return new Underline(mdsToInlines(md.children));
      case "abbreviation":
      case "break":
      case "crossReference":
      case "footnoteReference":
      case "html":
      case "image":
      case "imageReference":
      case "inlineCode":
      case "inlineMath":
      case "link":
      case "linkReference":
      case "subscript":
      case "superscript":
        throw new Error(`mdast inline type not yet implemented: ${md.type}`);
    }
  });
}
