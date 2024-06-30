import { EncodeInfo } from "@stencila/plugin";
import { Node } from "@stencila/types";
import { MySTEncodeContext } from "./encoder.js";
import { encodeNode } from "./node.js";

/**
 * Encode a Stencila Schema `Node` to a MyST string
 */
export function encode(node: Node): [string, EncodeInfo] {
  let context = new MySTEncodeContext();

  encodeNode(node, context)

  return [context.content, {mapping: context.mapping}];
}
