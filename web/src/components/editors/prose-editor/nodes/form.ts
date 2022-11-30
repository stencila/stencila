import { Attrs, Node, NodeSpec, ParseRule } from 'prosemirror-model'
import { EditorView } from 'prosemirror-view'

import StencilaForm from '../../../nodes/form'
import { StencilaExecutableView, executableAttrs } from './executable'

export function form(): NodeSpec {
  return {
    group: 'BlockContent',
    // Use +, rather than *, here so that if the `For` has no content
    // that at least a empty placeholder paragraph will be available for user to edit
    content: 'BlockContent+',
    // Necessary for copy/paste-ability of whole node, not just its content
    defining: true,
    attrs: {
      ...executableAttrs,
      deriveFrom: { default: null },
      deriveAction: { default: null },
      deriveItem: { default: null },
    },
    parseDOM,
    toDOM,
  }
}

export class StencilaFormView extends StencilaExecutableView<StencilaForm> {
  constructor(node: Node, view: EditorView, getPos: () => number) {
    super(node, view, getPos, getAttrs, toDOM)
  }
}

const parseDOM: ParseRule[] = [
  {
    tag: 'stencila-form',
    getAttrs,
    contentElement: '[slot=content]',
  },
]

function getAttrs(node: StencilaForm): Attrs {
  return {
    id: node.id,
    deriveFrom: node.getAttribute('derive-from'),
    deriveAction: node.getAttribute('derive-action'),
    deriveItem: node.getAttribute('derive-item'),
    errors: node.querySelector('[slot=errors]')?.innerHTML ?? '',
  }
}

function toDOM(node: Node) {
  const dom = document.createElement('stencila-form')
  dom.draggable = true
  dom.id = node.attrs.id
  if (node.attrs.deriveFrom)
    dom.setAttribute('derive-from', node.attrs.deriveFrom)
  if (node.attrs.deriveAction)
    dom.setAttribute('derive-action', node.attrs.deriveAction)
  if (node.attrs.deriveItem)
    dom.setAttribute('derive-item', node.attrs.deriveItem)

  const errors = document.createElement('div')
  errors.slot = 'errors'
  errors.innerHTML = node.attrs.errors
  errors.contentEditable = 'false'
  dom.appendChild(errors)

  const contentDOM = document.createElement('div')
  contentDOM.slot = 'content'
  dom.appendChild(contentDOM)

  return { dom, contentDOM }
}
