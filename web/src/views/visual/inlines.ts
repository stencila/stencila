import { NodeSpec, attrsParseToDOM } from "./prelude";

/**
 * A ProseMirror `NodeSpec` for a Stencila `Text` node
 */
export const Text: NodeSpec = {
  group: "Inline",
  inline: true,
  content: "text*",
  marks: "",
  ...attrsParseToDOM("span", "id"),
};

export const inlines = {
  Text,
  // Every schema needs to have a "text" type with no attributes
  text: { group: "Inline" },
};
