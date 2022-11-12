import { sentenceCase } from 'change-case'
import { Attrs, Node, NodeSpec, ParseRule } from 'prosemirror-model'
import { EditorView } from 'prosemirror-view'
import StencilaButton from '../../../nodes/button'
import {
  StencilaCodeExecutableView,
  codeExecutableAttrs,
} from './code-executable'

export function button(): NodeSpec {
  return {
    group: 'InlineContent',
    inline: true,
    attrs: {
      ...codeExecutableAttrs,
      name: { default: '' },
      label: { default: null },
    },
    parseDOM,
    toDOM,
  }
}

export class StencilaButtonView extends StencilaCodeExecutableView<StencilaButton> {
  constructor(node: Node, view: EditorView, getPos: () => number) {
    super(node, view, getPos, getAttrs, toDOM)
  }
}

const parseDOM: ParseRule[] = [
  {
    tag: 'stencila-button',
    getAttrs,
  },
]

function getAttrs(node: StencilaButton): Attrs {
  return {
    id: node.id,
    name: node.getAttribute('name') ?? undefined,
    label: node.getAttribute('label') ?? undefined,
    text: node.getAttribute('text') ?? undefined,
    programmingLanguage: node.getAttribute('programming-language') ?? undefined,
    guessLanguage: node.getAttribute('guess-language') ?? undefined,
  }
}

function toDOM(node: Node) {
  const dom = document.createElement('stencila-button')
  dom.draggable = true
  dom.id = node.attrs.id
  dom.setAttribute('name', node.attrs.name)
  dom.setAttribute('label', node.attrs.label ?? sentenceCase(node.attrs.name))
  dom.setAttribute('text', node.attrs.text)
  dom.setAttribute('programming-language', node.attrs.programmingLanguage)
  dom.setAttribute('guess-language', node.attrs.guessLanguage)

  return { dom }
}
