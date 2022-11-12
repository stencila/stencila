import { Attrs, Node, NodeSpec, ParseRule } from 'prosemirror-model'
import { EditorView } from 'prosemirror-view'
import StencilaSpan from '../../../nodes/span'
import { StencilaStyledView, styledAttrs } from './styled'

export function span(): NodeSpec {
  return {
    group: 'InlineContent',
    content: 'InlineContent*',
    inline: true,
    attrs: styledAttrs,
    parseDOM,
    toDOM,
  }
}

export class StencilaSpanView extends StencilaStyledView<StencilaSpan> {
  constructor(node: Node, view: EditorView, getPos: () => number) {
    super(node, view, getPos, getAttrs, toDOM)
  }
}

const parseDOM: ParseRule[] = [
  {
    tag: 'stencila-span',
    getAttrs,
    contentElement: '[slot=content]',
  },
]

function getAttrs(node: StencilaSpan): Attrs {
  return {
    id: node.id,
    programmingLanguage: node.getAttribute('programming-language'),
    guessLanguage: node.getAttribute('guess-language'),
    text: node.querySelector('[slot=text]')?.innerHTML,
    css: node.querySelector('[slot=css]')?.innerHTML,
    errors: node.querySelector('[slot=errors]')?.innerHTML,
  }
}

function toDOM(node: Node) {
  const dom = document.createElement('stencila-span')
  dom.draggable = true
  dom.id = node.attrs.id
  dom.setAttribute('programming-language', node.attrs.programmingLanguage)
  dom.setAttribute('guess-language', node.attrs.guessLanguage)

  const text = document.createElement('code')
  text.slot = 'text'
  text.innerHTML = node.attrs.text
  text.contentEditable = 'false'
  dom.appendChild(text)

  const css = document.createElement('code')
  css.slot = 'css'
  css.innerHTML = node.attrs.css
  css.contentEditable = 'false'
  dom.appendChild(css)

  const errors = document.createElement('span')
  errors.slot = 'errors'
  errors.innerHTML = node.attrs.errors
  errors.contentEditable = 'false'
  dom.appendChild(errors)

  const contentDOM = document.createElement('span')
  contentDOM.slot = 'content'
  dom.appendChild(contentDOM)

  return { dom, contentDOM }
}
