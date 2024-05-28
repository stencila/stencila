import { Inline } from "@stencila/types";
import { MySTEncodeContext } from "./encoder.js";

/**
 * Encode an array of Stencila `Inline` nodes to MyST
 */
export function encodeInlines(inlines: Inline[], context: MySTEncodeContext) {
  for (const inline of inlines) {
    encodeInline(inline, context);
  }
}

/**
 * Encode a Stencila `Inline` node to MyST
 */
export function encodeInline(inline: Inline, context: MySTEncodeContext) {
  if (inline == null) {
    context.pushString("null");
    return;
  } else if (typeof inline === "boolean" || typeof inline === "number") {
    context.pushString(inline.toString());
    return;
  }

  context.enterNode(inline.type, inline.id ?? "");

  switch (inline.type) {
    case "Text": {
      context.pushString(inline.value);
    }
  }

  context.exitNode();
}
