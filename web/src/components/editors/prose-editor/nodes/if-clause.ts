import { Attrs, Node, NodeSpec, ParseRule } from 'prosemirror-model'
import { EditorView } from 'prosemirror-view'

import StencilaIfClause from '../../../nodes/if-clause'
import {
  StencilaCodeExecutableView,
  codeExecutableAttrs,
} from './code-executable'

export function ifClause(): NodeSpec {
  return {
    // Use +, rather than *, here so that if the `If` has no content
    // that at least a empty placeholder paragraph will be available for user to edit
    content: 'BlockContent+',
    defining: true,
    isolating: true,
    // Not draggable, users can use up/down buttons instead
    draggable: false,
    attrs: codeExecutableAttrs,
    parseDOM,
    toDOM,
  }
}

export class StencilaIfClauseView extends StencilaCodeExecutableView<StencilaIfClause> {
  constructor(node: Node, view: EditorView, getPos: () => number) {
    super(node, view, getPos, getAttrs, toDOM)
  }
}

const parseDOM: ParseRule[] = [
  {
    tag: 'stencila-if-clause',
    getAttrs,
    contentElement: '[slot=content]',
  },
]

function getAttrs(node: StencilaIfClause): Attrs {
  return {
    id: node.id,
    programmingLanguage: node.getAttribute('programming-language'),
    guessLanguage: node.getAttribute('guess-language'),
    text: node.querySelector('[slot=text]')?.innerHTML ?? '',
    errors: node.querySelector('[slot=errors]')?.innerHTML ?? '',
  }
}

function toDOM(node: Node) {
  const dom = document.createElement('stencila-if-clause')
  dom.id = node.attrs.id
  dom.setAttribute('programming-language', node.attrs.programmingLanguage)
  if (node.attrs.guessLanguage)
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

  return { dom, contentDOM }
}
