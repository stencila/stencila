import { Attrs, Node, NodeSpec, ParseRule } from 'prosemirror-model'
import { EditorView } from 'prosemirror-view'
import StencilaCodeChunk from '../../../nodes/code-chunk'
import {
  codeExecutableAttrs,
  StencilaCodeExecutableView,
} from './code-executable'

export function codeChunk(): NodeSpec {
  return {
    group: 'BlockContent',
    attrs: { ...codeExecutableAttrs, outputs: { default: '' } },
    parseDOM,
    toDOM,
  }
}

export class StencilaCodeChunkView extends StencilaCodeExecutableView<StencilaCodeChunk> {
  constructor(node: Node, view: EditorView, getPos: () => number) {
    super(node, view, getPos, getAttrs, toDOM)
  }
}

const parseDOM: ParseRule[] = [
  {
    tag: 'stencila-code-chunk',
    getAttrs,
  },
]

function getAttrs(node: StencilaCodeChunk): Attrs {
  return {
    id: node.id,
    programmingLanguage: node.getAttribute('programming-language'),
    guessLanguage: node.getAttribute('guess-language'),
    text: node.querySelector('[slot=text]')?.innerHTML ?? '',
    errors: node.querySelector('[slot=errors]')?.innerHTML,
    outputs: node.querySelector('[slot=outputs]')?.innerHTML,
  }
}

function toDOM(node: Node) {
  const dom = document.createElement('stencila-code-chunk')
  dom.draggable = true
  dom.id = node.attrs.id
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

  if (node.attrs.outputs) {
    const outputs = document.createElement('div')
    outputs.slot = 'outputs'
    outputs.innerHTML = node.attrs.outputs
    outputs.contentEditable = 'false'
    dom.appendChild(outputs)
  }

  return { dom }
}
