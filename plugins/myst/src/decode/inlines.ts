import {
  CodeInline,
  Emphasis,
  ImageObject,
  Inline,
  Link,
  MathInline,
  Strong,
  Subscript,
  Superscript,
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
  return mds.map((md) => mdToInline(md));
}

export function mdToInline(md: PhrasingContent): Inline {
  {
    switch (md.type) {
      case "text":
        return new Text(md.value);
      case "emphasis":
        return new Emphasis(mdsToInlines(md.children));
      case "strong":
        return new Strong(mdsToInlines(md.children));
      case "inlineCode":
        return new CodeInline(md.value);
      case "underline":
        return new Underline(mdsToInlines(md.children));
      case "image":
        return new ImageObject(md.url);
      case "link":
        return new Link(mdsToInlines(md.children), md.url);
      case "inlineMath":
        return new MathInline(md.value);
      case "subscript":
        return new Subscript(mdsToInlines(md.children));
      case "superscript":
        return new Superscript(mdsToInlines(md.children));
      case "mystRole":
      // Roles should not exist after basicTransformations() in index.ts
      case "abbreviation":
      case "break":
      case "html":
      case "crossReference":
      case "footnoteReference":
      case "imageReference":
      case "linkReference":
        throw new Error(`mdast inline type not yet implemented: ${md.type}`);
    }
  }
}
