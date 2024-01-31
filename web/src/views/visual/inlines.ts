import {
  type NodeSpec,
  Node,
  NodeViewConstructor,
  getAttrs,
  attrsParseToDOM,
  attrsWithDefault,
  executableAttrs,
} from './prelude'

/**
 * A ProseMirror `NodeSpec` for a Stencila `InstructionInline`
 */
const InstructionInline: NodeSpec = {
  group: 'Inline',
  content: 'Inline*',
  inline: true,
  attrs: attrsWithDefault(
    null,
    'messages',
    'candidates',
    'assignee',
    'authors',
    'suggestion',
    ...executableAttrs
  ),
  parseDOM: [
    {
      tag: 'stencila-instruction-block',
      contentElement: '[slot=content]',
      getAttrs: (elem: HTMLElement) => ({
        messages: elem.querySelector('[slot=messages]')?.innerHTML,
        authors: elem.querySelector('[slot=authors]')?.innerHTML,
        suggestion: elem.querySelector('[slot=suggestion]')?.innerHTML,
        ...getAttrs('candidates', 'assignee', ...executableAttrs)(elem),
      }),
    },
  ],
  toDOM: (node: Node) => {
    const dom = document.createElement('stencila-instruction-block')
    dom.draggable = true
    dom.id = node.attrs.id
    dom.setAttribute('candidates', node.attrs.candidates)
    dom.setAttribute('assignee', node.attrs.assignee)

    const contentDOM = document.createElement('span')
    contentDOM.slot = 'content'
    dom.appendChild(contentDOM)

    if (node.attrs.messages) {
      const messages = document.createElement('span')
      messages.slot = 'messages'
      messages.innerHTML = node.attrs.messages
      dom.appendChild(messages)
    }

    if (node.attrs.authors) {
      const authors = document.createElement('span')
      authors.slot = 'authors'
      authors.innerHTML = node.attrs.authors
      dom.appendChild(authors)
    }

    if (node.attrs.suggestion) {
      const suggestion = document.createElement('span')
      suggestion.slot = 'suggestion'
      suggestion.innerHTML = node.attrs.suggestion
      dom.appendChild(suggestion)
    }

    return { dom, contentDOM }
  },
}

/**
 * A ProseMirror `NodeSpec` for a Stencila `Parameter` node
 */
const Parameter: NodeSpec = {
  group: 'Inline',
  inline: true,
  atom: true,
  ...attrsParseToDOM('stencila-parameter', 'id'),
}

/**
 * A ProseMirror `NodeSpec` for a Stencila `Text` node
 */
const Text: NodeSpec = {
  group: 'Inline',
  inline: true,
  content: 'text*',
  marks: '',
  ...attrsParseToDOM('stencila-text', 'id'),
}

// Export specs and views

export const specs: Record<string, NodeSpec> = {
  InstructionInline,
  Parameter,
  Text,
  // Every schema needs to have a "text" type with no attributes
  text: { group: 'Inline' },
}

export const views: Record<string, NodeViewConstructor> = {}
