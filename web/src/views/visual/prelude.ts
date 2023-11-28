import { Attrs, DOMOutputSpec, Node } from "prosemirror-model";

export { NodeSpec } from "prosemirror-model";

export const parseDOM = (tag: string, ...attrs: string[]) => [
  {
    tag,
    getAttrs: getAttrs(...attrs),
  },
];

export const getAttrs =
  (...attrs: string[]) =>
  (elem: HTMLElement | string): false | Attrs =>
    typeof elem === "string"
      ? false
      : Object.fromEntries(
          attrs.map((attr) => [attr, elem.getAttribute(attr)])
        );

export const toDOMAttrs = (node: Node, ...attrs: string[]) =>
  Object.fromEntries(attrs.map((attr) => [attr, node.attrs[attr]]));

export const toDOM =
  (tag: string, ...attrs: string[]) =>
  (node: Node): DOMOutputSpec =>
    [tag, toDOMAttrs(node, ...attrs), 0];

export const parseToDOM = (tag: string, ...attrs: string[]) => ({
  parseDOM: parseDOM(tag, ...attrs),
  toDOM: toDOM(tag, ...attrs),
});

export const attrsParseToDOM = (tag: string, ...attrs: string[]) => ({
  attrs: Object.fromEntries(attrs.map((attr) => [attr, {}])),
  ...parseToDOM(tag, ...attrs),
});
