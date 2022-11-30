import { Attrs, Node, NodeSpec, ParseRule } from 'prosemirror-model'
import { EditorView } from 'prosemirror-view'
import StencilaCodeBlock from '../../../nodes/code-block'
import { codeStaticAttrs, StencilaCodeStaticView } from './code-static'

export function codeBlock(): NodeSpec {
  return {
    group: 'BlockContent',
    attrs: codeStaticAttrs,
    parseDOM,
    toDOM,
  }
}

export class StencilaCodeBlockView extends StencilaCodeStaticView<StencilaCodeBlock> {
  constructor(node: Node, view: EditorView, getPos: () => number) {
    super(node, view, getPos, getAttrs, toDOM)
  }
}

const parseDOM: ParseRule[] = [
  {
    tag: 'stencila-code-block',
    getAttrs,
  },
]

function getAttrs(node: StencilaCodeBlock): Attrs {
  return {
    id: node.id,
    programmingLanguage: node.getAttribute('programming-language') ?? '',
    code: node.querySelector('[slot=code]')?.innerHTML ?? '',
  }
}

function toDOM(node: Node) {
  const dom = document.createElement('stencila-code-block')
  dom.draggable = true
  dom.id = node.attrs.id
  dom.setAttribute('programming-language', node.attrs.programmingLanguage)

  const code = document.createElement('pre')
  code.slot = 'code'
  code.innerHTML = node.attrs.code
  code.contentEditable = 'false'
  dom.appendChild(code)

  return { dom }
}
