import { Attrs, Node, NodeSpec, ParseRule } from 'prosemirror-model'
import { EditorView } from 'prosemirror-view'

import StencilaFor from '../../../nodes/for'
import {
  StencilaCodeExecutableView,
  codeExecutableAttrs,
} from './code-executable'

export function for_(): NodeSpec {
  return {
    group: 'BlockContent',
    // Use +, rather than *, here so that if the `For` has no content
    // that at least a empty placeholder paragraph will be available for user to edit
    content: 'BlockContent+',
    defining: true,
    isolating: true,
    draggable: true,
    attrs: {
      ...codeExecutableAttrs,
      symbol: { default: null },
      otherwise: { default: '' },
      iterations: { default: '' },
    },
    parseDOM,
    toDOM,
  }
}

export class StencilaForView extends StencilaCodeExecutableView<StencilaFor> {
  constructor(node: Node, view: EditorView, getPos: () => number) {
    super(node, view, getPos, getAttrs, toDOM)
  }
}

const parseDOM: ParseRule[] = [
  {
    tag: 'stencila-for',
    getAttrs,
    contentElement: '[slot=content]',
    consuming: true,
  },
]

function getAttrs(node: StencilaFor): Attrs {
  return {
    id: node.id,
    symbol: node.getAttribute('symbol'),
    programmingLanguage: node.getAttribute('programming-language'),
    guessLanguage: node.getAttribute('guess-language'),
    text: node.querySelector('[slot=text]')?.innerHTML ?? '',
    errors: node.querySelector('[slot=errors]')?.innerHTML ?? '',
    otherwise: node.querySelector('[slot=otherwise]')?.innerHTML ?? '',
    iterations: node.querySelector('[slot=iterations]')?.innerHTML ?? '',
  }
}

function toDOM(node: Node) {
  const dom = document.createElement('stencila-for')
  dom.draggable = true
  dom.id = node.attrs.id
  dom.setAttribute('symbol', node.attrs.symbol)
  dom.setAttribute('programming-language', node.attrs.programmingLanguage)
  dom.setAttribute('guess-language', node.attrs.guessLanguage)

  const text = document.createElement('pre')
  text.slot = 'text'
  text.innerHTML = node.attrs.text
  text.contentEditable = 'false'
  dom.appendChild(text)

  const errors = document.createElement('div')
  errors.slot = 'errors'
  errors.innerHTML = node.attrs.errors
  errors.contentEditable = 'false'
  dom.appendChild(errors)

  const contentDOM = document.createElement('div')
  contentDOM.slot = 'content'
  dom.appendChild(contentDOM)

  // Currently, the `otherwise` property is not editable, so add opacity
  // to indicate that is the case.
  const otherwise = document.createElement('div')
  otherwise.slot = 'otherwise'
  otherwise.innerHTML = node.attrs.otherwise
  otherwise.contentEditable = 'false'
  otherwise.setAttribute('style', 'opacity: 0.7;')
  dom.appendChild(otherwise)

  const iterations = document.createElement('div')
  iterations.slot = 'iterations'
  iterations.innerHTML = node.attrs.iterations
  iterations.contentEditable = 'false'
  dom.appendChild(iterations)

  return { dom, contentDOM }
}
