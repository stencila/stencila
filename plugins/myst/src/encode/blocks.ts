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
  const parentType = parentNodeType(context);

  switch (block.type) {
    case "Paragraph":
      encodeInlines(block.content, context);
      if (parentType === "Table") {
        // No line break in table cell.
        // Note, we currently do not push TableCell/Row on context node stack.
      } else if (parentType === "ListItem") {
        context.pushString("\n");
      } else {
        context.pushString("\n\n");
      }
      break;
    case "CodeBlock":
      context.pushString("```" + block.programmingLanguage + "\n");
      context.pushString(block.code + "\n");
      context.pushString("```\n\n");
      break;
    case "CodeChunk":
      context.pushString("```{code-cell} " + block.programmingLanguage + "\n");
      context.pushString(block.code + "\n");
      context.pushString("```\n\n");
      break;
    case "IncludeBlock":
      context.pushString("```{embed} " + block.source + "\n");
      context.pushString("```\n\n");
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
      // Disallow nested Figure
      if (parentType != "Figure") {
        if (
          block.content[0].type === "Table" ||
          (block.content[0]?.type === "Figure" &&
            block.content[0]?.content[0]?.type === "Paragraph" &&
            block.content[1]?.type === "Table")
        ) {
          context.pushString(":::{table}\n");
        } else {
          context.pushString(":::{figure}\n");
        }
        if (block.name) {
          context.pushString(":name: " + block.name + "\n");
        }
        if (block.description) {
          context.pushString(":alt: " + block.description + "\n");
        }
      }
      encodeBlocks(block.content, context);
      if (parentType != "Figure") {
        context.pushString(":::\n\n");
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
      context.pushString(":::\n\n");
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
      context.pushString("\n");
      break;
    case "Claim":
      context.pushString(":::{prf:" + toMySTProofKind[block.claimType] + "}\n");
      encodeBlocks(block.content, context);
      context.pushString(":::\n\n");
      break;
    case "QuoteBlock":
      context.pushString(":::{blockquote}\n");
      encodeBlocks(block.content, context);
      // Currently only support Text citation
      if (block.cite?.type === "Text") {
        context.pushString("-- " + block.cite.value + "\n");
      }
      context.pushString(":::\n\n");
      break;
    case "Section":
      context.pushString("+++");
      if (block.sectionType) {
        context.pushString(
          ' { "part": "' + block.sectionType.toLowerCase() + '" }'
        );
      }
      context.pushString("\n");
      encodeBlocks(block.content, context);
      context.pushString("+++\n\n");
      break;
    case "CallBlock":
    case "DeleteBlock":
    case "ForBlock":
    case "Form":
    case "IfBlock":
    case "InsertBlock":
    case "InstructionBlock":
    case "ModifyBlock":
    case "ReplaceBlock":
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

const toMySTProofKind = {
  Statement: "proposition",
  Theorem: "theorem",
  Lemma: "lemma",
  Proof: "proof",
  Postulate: "axiom",
  Hypothesis: "conjecture",
  Proposition: "proposition",
  Corollary: "corollary",
};
