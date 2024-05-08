import {
  CodeInline,
  Emphasis,
  Inline,
  Strong,
  Text,
  Underline,
} from "@stencila/types";
import type { PhrasingContent } from "mdast";

/**
 * An interface for a MyST role https://mystmd.org/spec/overview#roles
 *
 * TODO: There is probably a published type for this amongst the MyST packages.
 * (and for MyST directives). This is just an initial stab to get things type checking,
 * look for published type, and if not available implement one based on above spec.
 */
interface MySTRole {
  type: "mystRole";
  name: string;
  value: string;
}

type MySTInline = PhrasingContent | MySTRole;

/**
 * Transform MDAST `PhrasingContent` to Stencila Schema `Inline` nodes
 *
 * This is equivalent to the Rust `mds_to_inlines` function in
 * `rust/codec-markdown/src/decode/inlines.rs`.
 *
 * This is also an update of code in
 * https://github.com/stencila/encoda/blob/master/src/codecs/md/index.ts.
 */
export function mdsToInlines(mds: MySTInline[]): Inline[] {
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
        return mystRole(md);
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
      default:
        throw new Error(`mdast inline type not yet implemented: ${md.type}`);
    }
  });
}

/**
 * Transform a `MySTRole` into a Stencila `Inline` node
 */
function mystRole(inline: MySTRole): Inline {
  switch (inline.name) {
    case "u":
    case "underline":
      return new Underline([new Text(inline.value)]);
    default:
      throw new Error(`mystRole not yet implemented: ${inline.name}`);
  }
}
