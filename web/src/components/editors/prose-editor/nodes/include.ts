import { Attrs, Node, NodeSpec, ParseRule } from 'prosemirror-model'
import { EditorView } from 'prosemirror-view'

import StencilaInclude from '../../../nodes/include'
import { StencilaExecutableView, executableAttrs } from './executable'

export const includeAttrs = {
  ...executableAttrs,
  source: { default: '' },
  select: { default: null },
  content: { default: null },
}

export function include(): NodeSpec {
  return {
    group: 'BlockContent',
    attrs: includeAttrs,
    parseDOM,
    toDOM,
  }
}

export class StencilaIncludeView extends StencilaExecutableView<StencilaInclude> {
  constructor(node: Node, view: EditorView, getPos: () => number) {
    super(node, view, getPos, getAttrs, toDOM)
  }
}

const parseDOM: ParseRule[] = [
  {
    tag: 'stencila-include',
    getAttrs,
  },
]

function getAttrs(node: StencilaInclude): Attrs {
  return {
    id: node.id,
    source: node.getAttribute('source'),
    select: node.getAttribute('select'),
    errors: node.querySelector('[slot=errors]')?.innerHTML,
    content: node.querySelector('[slot=content]')?.innerHTML,
  }
}

function toDOM(node: Node) {
  const dom = document.createElement('stencila-include')
  dom.draggable = true
  dom.id = node.attrs.id
  dom.setAttribute('source', node.attrs.source)
  dom.setAttribute('select', node.attrs.select)

  const errors = document.createElement('div')
  errors.slot = 'errors'
  errors.innerHTML = node.attrs.errors
  errors.contentEditable = 'false'
  dom.appendChild(errors)

  // Note the `content` property is not editable so we just store it
  // on the node as HTML, not as a `contentDOM`, and reduce it's opacity
  const content = document.createElement('div')
  content.slot = 'content'
  content.innerHTML = node.attrs.content
  content.contentEditable = 'false'
  content.setAttribute('style', 'opacity: 0.75;')
  dom.appendChild(content)

  return { dom }
}
