import { NodeSpec, attrsParseToDOM, toDOMAttrs, getAttrs } from "./prelude";

/**
 * A ProseMirror `NodeSpec` for a Stencila `Heading`
 *
 * Note that, consistent with treatment elsewhere, `h2` => level 3 etc.
 * This is because there should only be one `h1` (for the title) and when encoding to
 * HTML we add one to the level e.g. `level: 1` => `h2`
 */
export const Heading: NodeSpec = {
  group: "Block",
  content: "Inline*",
  marks: "_",
  attrs: {
    level: { default: 1 },
    id: { default: null },
  },
  parseDOM: [
    {
      tag: "h1",
      getAttrs: (elem: HTMLElement) => ({ level: 1, ...getAttrs("id")(elem) }),
    },
    {
      tag: "h2",
      getAttrs: (elem: HTMLElement) => ({ level: 1, ...getAttrs("id")(elem) }),
    },
    {
      tag: "h3",
      getAttrs: (elem: HTMLElement) => ({ level: 2, ...getAttrs("id")(elem) }),
    },
    {
      tag: "h4",
      getAttrs: (elem: HTMLElement) => ({ level: 3, ...getAttrs("id")(elem) }),
    },
    {
      tag: "h5",
      getAttrs: (elem: HTMLElement) => ({ level: 4, ...getAttrs("id")(elem) }),
    },
    {
      tag: "h6",
      getAttrs: (elem: HTMLElement) => ({ level: 5, ...getAttrs("id")(elem) }),
    },
  ],
  toDOM(node) {
    return [`h${(node.attrs.level as number) + 1}`, toDOMAttrs(node, "id"), 0];
  },
};

/**
 * A ProseMirror `NodeSpec` for a Stencila `Paragraph`
 */
export const Paragraph: NodeSpec = {
  group: "Block",
  content: "Inline*",
  marks: "_",
  ...attrsParseToDOM("p", "id"),
};

export const blocks = {
  Heading,
  Paragraph,
};
