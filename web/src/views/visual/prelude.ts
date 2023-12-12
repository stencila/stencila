import {
  type Attrs,
  type DOMOutputSpec,
  Node,
  ParseRule,
} from 'prosemirror-model'
import { type NodeView as NodeViewInterface } from 'prosemirror-view'

export { type NodeSpec, Node } from 'prosemirror-model'
export { type NodeViewConstructor } from 'prosemirror-view'

// TODO: Document this module

export const executableAttrs = ['id']

export const codeExecutableAttrs = [
  ...executableAttrs,
  'code',
  'programming-language',
]

export const attrsWithDefault = (default_: unknown, ...attrs: string[]) =>
  Object.fromEntries(attrs.map((attr) => [attr, { default: default_ }]))

export const getAttrs =
  (...attrs: string[]) =>
  (elem: HTMLElement | string): false | Attrs =>
    typeof elem === 'string'
      ? false
      : Object.fromEntries(attrs.map((attr) => [attr, elem.getAttribute(attr)]))

export const parseDOM = (tag: string, ...attrs: string[]) => [
  {
    tag,
    getAttrs: getAttrs(...attrs),
  },
]

export const parseDOMWithContent = (
  tag: string,
  contentElement: string,
  ...attrs: string[]
): ParseRule[] => [
  {
    tag,
    contentElement,
    getAttrs: getAttrs(...attrs),
  },
]

export const toDOMAttrs = (node: Node, ...attrs: string[]) =>
  Object.fromEntries(attrs.map((attr) => [attr, node.attrs[attr]]))

export const toDOM =
  (tag: string, ...attrs: string[]) =>
  (node: Node): DOMOutputSpec => [tag, toDOMAttrs(node, ...attrs), 0]

export const parseToDOM = (tag: string, ...attrs: string[]) => ({
  parseDOM: parseDOM(tag, ...attrs),
  toDOM: toDOM(tag, ...attrs),
})

export const attrsParseToDOM = (tag: string, ...attrs: string[]) => ({
  attrs: attrsWithDefault(null, ...attrs),
  ...parseToDOM(tag, ...attrs),
})

export class NodeView implements NodeViewInterface {
  dom: InstanceType<typeof window.Node>

  contentDOM?: HTMLElement

  /**
   * Prevent the editor view from trying to handle some or all DOM events that
   * bubble up from the node view
   *
   * This prevents all events apart from drag events (to enable drag and drop).
   * Without this, a lot of things break, in particular the toggling of node selection
   * and backspacing in <input>s. However, it is not clear if this blanket banning
   * affects the editing of ProseMirror content. A more selective approach may be
   * required (e.g. only ignoring events on inputs).
   *
   * @see https://discuss.prosemirror.net/t/creating-a-custom-node-with-inline-input/1282/5
   */
  stopEvent(event: Event) {
    return !event.type.startsWith('drag')
  }
}
