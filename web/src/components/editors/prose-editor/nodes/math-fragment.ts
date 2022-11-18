import { Attrs, Node, NodeSpec, ParseRule } from 'prosemirror-model'
import { EditorView } from 'prosemirror-view'
import StencilaMathFragment from '../../../nodes/math-fragment'
import { StencilaMathView, mathAttrs } from './math'

export function mathFragment(): NodeSpec {
  return {
    group: 'InlineContent',
    inline: true,
    attrs: mathAttrs,
    parseDOM,
    toDOM,
  }
}

export class StencilaMathFragmentView extends StencilaMathView<StencilaMathFragment> {
  constructor(node: Node, view: EditorView, getPos: () => number) {
    super(node, view, getPos, getAttrs, toDOM)
  }
}

const parseDOM: ParseRule[] = [
  {
    tag: 'stencila-math-fragment',
    getAttrs,
  },
]

function getAttrs(node: StencilaMathFragment): Attrs {
  return {
    id: node.id,
    mathLanguage: node.getAttribute('math-language'),
    code: node.querySelector('[slot=code]')?.innerHTML,
    errors: node.querySelector('[slot=errors]')?.innerHTML,
    mathml: node.querySelector('[slot=mathml]')?.innerHTML,
  }
}

function toDOM(node: Node) {
  const dom = document.createElement('stencila-math-fragment')
  dom.draggable = true
  dom.id = node.attrs.id
  dom.setAttribute('math-language', node.attrs.mathLanguage)

  const code = document.createElement('code')
  code.slot = 'code'
  code.innerHTML = node.attrs.code
  code.contentEditable = 'false'
  dom.appendChild(code)

  const errors = document.createElement('span')
  errors.slot = 'errors'
  errors.innerHTML = node.attrs.errors
  errors.contentEditable = 'false'
  dom.appendChild(errors)

  const mathml = document.createElement('span')
  mathml.slot = 'mathml'
  mathml.innerHTML = node.attrs.mathml
  mathml.contentEditable = 'false'
  dom.appendChild(mathml)

  return { dom }
}
