import HtmlFragment from 'html-fragment'
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
      derivedFrom: { default: null },
      validator: {
        default: null,
      },
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
    label: node.getAttribute('label') ?? undefined,
    derivedFrom: node.getAttribute('derived-from') ?? undefined,
    errors: node.querySelector('[slot=errors]')?.innerHTML ?? '',
    validator: node.querySelector('[slot=validator]')?.outerHTML,
  }
}

function toDOM(node: Node) {
  const dom = document.createElement('stencila-parameter')
  dom.contentEditable = 'false'
  dom.draggable = true
  dom.id = node.attrs.id
  dom.setAttribute('name', node.attrs.name)
  dom.setAttribute('label', node.attrs.label)
  dom.setAttribute('derived-from', node.attrs.derivedFrom)

  const errors = document.createElement('div')
  errors.slot = 'errors'
  errors.innerHTML = node.attrs.errors
  errors.contentEditable = 'false'
  dom.appendChild(errors)

  if (node.attrs.validator) {
    const validator = HtmlFragment(node.attrs.validator).firstElementChild
    if (validator) {
      validator.setAttribute('slot', 'validator')
      dom.appendChild(validator)
    }
  }

  return { dom }
}
