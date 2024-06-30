import {
  Admonition,
  AdmonitionType,
  Block,
  CodeBlock,
  Figure,
  Heading,
  ImageObject,
  List,
  ListItem,
  MathBlock,
  Paragraph,
  QuoteBlock,
  Table,
  TableCell,
  TableRow,
  Text,
  ThematicBreak,
} from "@stencila/types";
import type {
  FlowContent,
  Block as MySTBlock,
  PhrasingContent,
  AdmonitionTitle,
} from "myst-spec";

import { mdToInline, mdsToInlines } from "./inlines.js";
/**
 * Transform MyST `Block` nodes to Stencila Schema `Block` nodes
 *
 * This is equivalent to the Rust `mds_to_blocks` function in
 * `rust/codec-markdown/src/decode/blocks.rs`.
 *
 * This is also an update of code in
 * https://github.com/stencila/encoda/blob/master/src/codecs/md/index.ts.
 */
export function mdsToBlocks(mds: (MySTBlock | FlowContent)[]): Block[] {
  return mds.map((md) => mdToBlock(md)).filter((_) => !!_) as Block[];
}

export function mdToBlock(md: MySTBlock | FlowContent): Block | undefined {
  // TODO: not sure what to use for md comments, just filter them out for now
  if ((md.type as string) === "comment") return undefined;

  switch (md.type) {
    case "block":
      // A MyST Block corresponds to a seperate cell in a notebook
      // But we are currently assuming only one at top level block, see index.ts
      throw new Error(`Expected only one top-level block`);
    case "mystDirective":
      throw new Error(
        "mystDirective should not exist after basicTransformations(): " +
          md.name +
          "\nIs it implemented in myst-parser or myst-transforms basicTransformations()?"
      );
    case "paragraph":
      return new Paragraph(mdsToInlines(md.children));
    case "heading":
      return new Heading(md.depth, mdsToInlines(md.children));
    case "code":
      return new CodeBlock(md.value, { programmingLanguage: md.lang });
    case "blockquote":
      return new QuoteBlock(md.children as Block[]);
    case "list":
      return new List(
        md.children.map((li) => {
          let previousInlines: Paragraph | null = null;
          return new ListItem(
            li.children
              .map((c) => {
                if (mystBlockTypes.includes(c.type)) {
                  previousInlines = null;
                  return mdToBlock(c as FlowContent | MySTBlock)!;
                } else {
                  // If a child is not a Stencila Block compatible type, then assume it's an Inline
                  const inlines = mdToInline(c as PhrasingContent);
                  if (previousInlines === null) {
                    // so wrap it in a Stencila Paragraph...
                    previousInlines = new Paragraph([inlines]);
                    return previousInlines;
                  } else {
                    // ...or add adjacent Inline's to the existing Paragraph
                    previousInlines.content.push(inlines);
                    return undefined;
                  }
                }
              })
              .filter((_) => !!_) as Block[],
            // `checked` is not defined, but actually does exist in the MyST ListItem checkbox
            { isChecked: (li as any).checked }
          );
        }),
        md.ordered ? "Ascending" : "Unordered"
      );
    case "admonition":
      const title = md.children?.filter(
        (c) => c.type === "admonitionTitle"
      ) as AdmonitionTitle[];
      const titleInlines = title?.[0]?.children as PhrasingContent[];
      return new Admonition(
        toStencilaAdmonitionKind[md.kind ?? "note"] as AdmonitionType,
        mdsToBlocks(
          md.children?.filter(
            (c) => c.type !== "admonitionTitle"
          ) as FlowContent[]
        ),
        {
          title: titleInlines ? mdsToInlines(titleInlines) : undefined,
          isFolded: md.class === "dropdown" ? true : undefined,
        }
      );
    case "container":
      switch (md.kind) {
        case "quote":
          return new QuoteBlock(
            mdsToBlocks(getBlocksFromFirstChildOfType("blockquote", md)),
            {
              cite: {
                type: "Text",
                value: getBlocksFromFirstChildOfType("caption", md)[0]
                  .children[0].value,
              },
            }
          );
      }

      return new Figure(
        md.children?.map((r) => {
          switch (r.type) {
            case "table":
              return mdToBlock(r)!;
            case "image":
              return new Paragraph([
                new ImageObject(r.url, {
                  description: r.alt ? new Text(r.alt) : undefined,
                }),
              ]);
            // TODO: Find out what the diff between these two are
            case "caption":
            case "legend":
              return new Figure(mdsToBlocks(r.children));
            case "blockquote":
              return new QuoteBlock(mdsToBlocks(r.children));
            default:
              throw new Error(
                `MyST container type not yet implemented: ${(r as any).type}`
              );
          }
        }),
        { name: md.identifier, label: md.label }
      );
    case "math":
      return new MathBlock(md.value, { label: md.label });
    case "table":
      return new Table(
        md.children.map(
          (r) =>
            new TableRow(
              r.children.map(
                (c) =>
                  new TableCell(
                    mdsToInlines(c.children).map((p) => new Paragraph([p])),
                    { cellType: c.header ? "HeaderCell" : "DataCell" }
                  )
              ),
              {
                // If all cells in the row are HeaderCells, then assume this is a HeaderRow
                rowType: !r.children.some((c) => !c.header)
                  ? "HeaderRow"
                  : undefined,
              }
            )
        )
      );
    case "thematicBreak":
      return new ThematicBreak();
    case "mystComment":
    case "definition":
    case "footnoteDefinition":
    case "html":
    case "mystTarget":
      throw new Error(`Not yet implemented: ${md.type}`);
  }
}

const toStencilaAdmonitionKind = {
  attention: "Important",
  caution: "Warning",
  danger: "Danger",
  error: "Error",
  hint: "Tip",
  important: "Important",
  note: "Note",
  seealso: "Note",
  tip: "Tip",
  warning: "Warning",
};

const mystBlockTypes = [
  "block",
  "mystDirective",
  "paragraph",
  "heading",
  "code",
  "blockquote",
  "list",
  "admonition",
  "container",
  "definition",
  "footnoteDefinition",
  "html",
  "math",
  "mystComment",
  "mystTarget",
  "table",
  "thematicBreak",
];

function getBlocksFromFirstChildOfType(
  type: string,
  md: MySTBlock | FlowContent
): (MySTBlock | FlowContent)[] {
  const child = md.children?.find((c) => c.type === type);
  if (child) {
    return child.children;
  } else {
    return [];
  }
}
