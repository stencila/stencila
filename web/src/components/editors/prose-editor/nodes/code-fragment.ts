import { Attrs, Node, NodeSpec, ParseRule } from 'prosemirror-model'
import { EditorView } from 'prosemirror-view'
import StencilaCodeFragment from '../../../nodes/code-fragment'
import { codeStaticAttrs, StencilaCodeStaticView } from './code-static'

export function codeFragment(): NodeSpec {
  return {
    group: 'InlineContent',
    inline: true,
    attrs: codeStaticAttrs,
    parseDOM,
    toDOM,
  }
}

export class StencilaCodeFragmentView extends StencilaCodeStaticView<StencilaCodeFragment> {
  constructor(node: Node, view: EditorView, getPos: () => number) {
    super(node, view, getPos, getAttrs, toDOM)
  }
}

const parseDOM: ParseRule[] = [
  {
    tag: 'stencila-code-fragment',
    getAttrs,
  },
]

function getAttrs(node: StencilaCodeFragment): Attrs {
  return {
    id: node.id,
    programmingLanguage: node.getAttribute('programming-language') ?? '',
    text: node.querySelector('[slot=text]')?.innerHTML,
  }
}

function toDOM(node: Node) {
  const dom = document.createElement('stencila-code-fragment')
  dom.draggable = true
  dom.id = node.attrs.id
  dom.setAttribute('programming-language', node.attrs.programmingLanguage)

  const text = document.createElement('code')
  text.slot = 'text'
  text.innerHTML = node.attrs.text
  text.contentEditable = 'false'
  dom.appendChild(text)

  return { dom }
}
