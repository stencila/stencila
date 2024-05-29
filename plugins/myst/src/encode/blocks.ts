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
    case "Paragraph":
      encodeInlines(block.content, context);
      if (parentNodeType(context) === "ListItem") {
        context.pushString("\n");
      } else {
        context.pushString("\n\n");
      }
      break;
    case "CodeBlock":
      context.pushString("```" + block.programmingLanguage + "\n");
      context.pushString(block.code + "\n");
      context.pushString("```\n");
      break;
    case "Heading":
      context.pushString("#".repeat(block.level) + " ");
      encodeInlines(block.content, context);
      context.pushString("\n\n");
      break;
    case "List":
      const level = listLevel(context);
      block.items.forEach((item, index) => {
        context.enterNode(item.type, item.id ?? "");
        let listMark =
          " ".repeat(level * 2) +
          (block.order === "Ascending"
            ? index + 1 + ". "
            : block.order === "Descending"
              ? block.items.length - index + 1 + ". "
              : "- ");
        if (item.isChecked === true) {
          listMark += "[x] ";
        } else if (item.isChecked === false) {
          listMark += "[ ] ";
        }
        context.pushString(listMark);
        item.content.forEach((b, i) => {
          if (i > 0 && b.type !== "List") {
            context.pushString(" ".repeat(listMark.length));
          }
          encodeBlock(b, context);
        });
        context.exitNode();
      });
      if (level === 0) {
        context.pushString("\n");
      }
      break;
    case "Admonition":
    case "CallBlock":
    case "Claim":
    case "CodeChunk":
    case "DeleteBlock":
    case "Figure":
    case "ForBlock":
    case "Form":
    case "IfBlock":
    case "IncludeBlock":
    case "InsertBlock":
    case "InstructionBlock":
    case "MathBlock":
    case "ModifyBlock":
    case "QuoteBlock":
    case "ReplaceBlock":
    case "Section":
    case "StyledBlock":
    case "Table":
    case "ThematicBreak":
    default:
      throw new Error(`Not yet implemented: ${block.type}`);
  }

  context.exitNode();
}

const parentNodeType = (context: MySTEncodeContext) =>
  context.nodeStack.at(-2)?.[0];

const listLevel = (context: MySTEncodeContext) =>
  context.nodeStack
    .slice(0, -1)
    .reduce((l, c) => (c[0] === "List" ? l + 1 : l), 0);
