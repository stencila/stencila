import { Attrs, Node, NodeSpec, ParseRule } from 'prosemirror-model'
import { EditorView } from 'prosemirror-view'

import StencilaCallArgument from '../../../nodes/call-argument'
import { StencilaExecutableView, executableAttrs } from './executable'

export function callArgument(): NodeSpec {
  return {
    // Not draggable
    draggable: false,
    attrs: {
      ...executableAttrs,
      name: { default: '' },
      programmingLanguage: { default: '' },
      guessLanguage: { default: null },
      text: { default: null },
      errors: { default: null },
    },
    parseDOM,
    toDOM,
  }
}

export class StencilaCallArgumentView extends StencilaExecutableView<StencilaCallArgument> {
  constructor(node: Node, view: EditorView, getPos: () => number) {
    super(node, view, getPos, getAttrs, toDOM)
  }
}

const parseDOM: ParseRule[] = [
  {
    tag: 'stencila-call-argument',
    getAttrs,
  },
]

function getAttrs(node: StencilaCallArgument): Attrs {
  return {
    id: node.id,
    name: node.getAttribute('name'),
    programmingLanguage: node.getAttribute('programming-language') ?? '',
    guessLanguage: node.getAttribute('guess-language'),
    text: node.querySelector('[slot=text]')?.innerHTML ?? '',
    errors: node.querySelector('[slot=errors]')?.innerHTML ?? null,
  }
}

function toDOM(node: Node) {
  const dom = document.createElement('stencila-call-argument')
  dom.id = node.attrs.id
  dom.setAttribute('name', node.attrs.name)
  dom.setAttribute('programming-language', node.attrs.programmingLanguage)
  if (node.attrs.guessLanguage) {
    dom.setAttribute('guess-language', node.attrs.guessLanguage)
  }

  const text = document.createElement('pre')
  text.slot = 'text'
  text.innerHTML = node.attrs.text
  text.contentEditable = 'false'
  dom.appendChild(text)

  if (node.attrs.errors) {
    const errors = document.createElement('div')
    errors.slot = 'errors'
    errors.innerHTML = node.attrs.errors
    errors.contentEditable = 'false'
    dom.appendChild(errors)
  }

  return { dom }
}
