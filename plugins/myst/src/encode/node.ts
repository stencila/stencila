import { Node } from "@stencila/types";

import { MySTEncodeContext } from "./encoder.js";
import { encodeBlocks } from "./blocks.js";

export function encodeNode(node: Node, context: MySTEncodeContext) {
  if (typeof node === "object" && node !== null && !Array.isArray(node)) {
    switch (node.type) {
      case "Article":
        context.enterNode(node.type, node.id ?? "");
        encodeBlocks(node.content, context);
        break;
      default:
        throw new Error(`Unhandled node type: ${node.type}`);
    }
    context.exitNode();
  } else {
    throw new Error(`Unhandled node type: ${node}`);
  }
}
