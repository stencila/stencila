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
      const parentType = parentNodeType(context);
      if (parentType === "Table") {
        // No line break in table cell. Note, we currently do not push TableCell/Row on stack.
      } else if (parentType === "ListItem") {
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
    case "Figure":
      // TODO: Seems there are four possible ways to encode a MyST Figure:
      //   caption: if a Paragraph block with a Text is first child of the Figure container
      //   legend: if a Paragraph block with a Text is first child of the Figure container
      //   table: if a Table block is first child of the Figure container after taking into account
      //   the above (in this case make first line `:::{table}` not `:::{figure}` )
      //   image: if a Paragraph block with an ImageObject is first child of the Figure container
      // Either implement rules as per above
      if (parentNodeType(context) != "Figure") {
        context.pushString(":::{figure}\n");
      }
      if (block.name) {
        context.pushString(":name: " + block.name + "\n");
      }
      if (block.label) {
        context.pushString(":label: " + block.label + "\n");
      }
      if (block.description) {
        context.pushString(":alt: " + block.description + "\n");
      }
      encodeBlocks(block.content, context);
      if (parentNodeType(context) != "Figure") {
        context.pushString(":::\n");
      }
      break;
    case "ThematicBreak":
      context.pushString("---\n");
      break;
    case "Admonition":
      context.pushString(
        ":::{" + toMySTAdmonitionKind[block.admonitionType] + "}"
      );
      if (block.title) {
        context.pushString(" ");
        encodeInlines(block.title, context);
      }
      context.pushString("\n");
      if (block.isFolded) {
        context.pushString(":class: dropdown\n");
      }
      encodeBlocks(block.content, context);
      context.pushString(":::\n");
      break;
    case "MathBlock":
      context.pushString("```{math}\n");
      if (block.label) {
        context.pushString(":label: " + block.label + "\n");
      }
      context.pushString(block.code);
      context.pushString("\n```\n");
      break;
    case "Table":
      block.rows.forEach((row, rowIndex) => {
        context.pushString("|");
        row.cells.forEach((cell, cellIndex) => {
          if (cellIndex > 0) {
            context.pushString(" |");
          }
          context.pushString(" ");
          encodeBlocks(cell.content, context);
          context.pushString(" ");
        });
        context.pushString("|\n");
        if (row.rowType === "HeaderRow") {
          context.pushString("|" + "--- |".repeat(row.cells.length) + "\n");
        }
      });
      break;

    case "CallBlock":
    case "Claim":
    case "CodeChunk":
    case "DeleteBlock":
    case "ForBlock":
    case "Form":
    case "IfBlock":
    case "IncludeBlock":
    case "InsertBlock":
    case "InstructionBlock":
    case "ModifyBlock":
    case "QuoteBlock":
    case "ReplaceBlock":
    case "Section":
    case "StyledBlock":
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

const toMySTAdmonitionKind = {
  Danger: "danger",
  Error: "error",
  Important: "important",
  Note: "note",
  seealso: "Info",
  Tip: "tip",
  Warning: "warning",
  Info: "note",
  Success: "note",
  Failure: "error",
};
