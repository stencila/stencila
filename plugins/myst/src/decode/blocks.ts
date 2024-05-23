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
      throw new Error(`Not yet implemented: ${md.type}`);
    case "mystDirective":
      // Directive should not exist after basicTransformations() in index.ts
      throw new Error(`Not yet implemented: ${md.type}`);
    case "paragraph":
      return new Paragraph(mdsToInlines(md.children));
    case "heading":
      return new Heading(md.depth, mdsToInlines(md.children));
    case "code":
      return new CodeBlock(md.value);
    case "blockquote":
      return new QuoteBlock(md.children as Block[]);
    case "list":
      return new List(
        md.children.map((li) => {
          return new ListItem(
            li.children.map((c) => {
              if (mystBlockTypes.includes(c.type)) {
                return mdToBlock(c as FlowContent | MySTBlock)!;
              } else {
                // If a child is not a Stencila Block compatible type, then assume it's an Inline
                // so wrap it in a Stencila Paragraph
                return new Paragraph([mdToInline(c as PhrasingContent)]);
              }
            }),
            // For some reason `checked` is not defined, but actually does exist in the ListItem
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
        { title: titleInlines ? mdsToInlines(titleInlines) : undefined }
      );
    case "container":
      return new Figure(
        md.children?.map((r) => {
          switch (r.type) {
            case "table":
              return mdToBlock(r)!;
            case "image":
              return new Paragraph([new ImageObject(r.url)]);
            // TODO: Figure out what the diff between these two are
            case "caption":
            case "legend":
              return new Figure(mdsToBlocks(r.children));
          }
        })
      );
    case "math":
      return new MathBlock(md.value);
    case "table":
      return new Table(
        md.children.map(
          (r) =>
            new TableRow(
              r.children.map(
                (c) =>
                  new TableCell(
                    mdsToInlines(c.children).map((p) => new Paragraph([p]))
                  )
              )
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
  seealso: "Info",
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
