import { Attrs, Node, NodeSpec, ParseRule } from 'prosemirror-model'
import { EditorView } from 'prosemirror-view'
import StencilaParameter from '../../../nodes/parameter'
import { StencilaExecutableView, executableAttrs } from './executable'

export function parameter(): NodeSpec {
  return {
    group: 'InlineContent',
    inline: true,
    draggable: true,
    attrs: {
      ...executableAttrs,
      name: { default: '' },
      label: { default: '' },
    },
    parseDOM,
    toDOM,
  }
}

export class StencilaParameterView extends StencilaExecutableView<StencilaParameter> {
  constructor(node: Node, view: EditorView, getPos: () => number) {
    super(node, view, getPos, getAttrs, toDOM)
  }
}

const parseDOM: ParseRule[] = [
  {
    tag: 'stencila-parameter',
    getAttrs,
  },
]

function getAttrs(node: StencilaParameter): Attrs {
  return {
    id: node.id,
    name: node.getAttribute('name'),
    label: node.getAttribute('label'),
  }
}

function toDOM(node: Node) {
  const dom = document.createElement('stencila-parameter')
  dom.draggable = true
  dom.id = node.attrs.id
  dom.setAttribute('name', node.attrs.name)
  dom.setAttribute('label', node.attrs.label)

  return { dom }
}
