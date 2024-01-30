import {
  type NodeSpec,
  Node,
  NodeViewConstructor,
  getAttrs,
  attrsWithDefault,
  parseDOMWithContent,
  codeExecutableAttrs,
  executableAttrs,
} from './prelude'

/**
 * A ProseMirror `NodeSpec` for a Stencila `Heading`
 *
 * Note that, consistent with treatment elsewhere, `h2` => level 3 etc.
 * This is because there should only be one `h1` (for the title) and when encoding to
 * HTML we add one to the level e.g. `level: 1` => `h2`
 */
const Heading: NodeSpec = {
  group: 'Block',
  content: 'Inline*',
  marks: '_',
  attrs: {
    id: { default: null },
    level: { default: 1 },
    authors: { default: null },
  },
  parseDOM: [
    // For parsing Stencila DOM HTML
    {
      tag: 'stencila-heading',
      contentElement: '[slot=content]',
      getAttrs: (elem: HTMLElement) => ({
        id: elem.getAttribute('id'),
        level: parseInt(elem.getAttribute('level')),
        authors: elem.querySelector('[slot=authors]')?.innerHTML,
      }),
    },
    // For parsing any old HTML...
    {
      tag: 'h1',
      getAttrs: () => ({ level: 1 }),
    },
    {
      tag: 'h2',
      getAttrs: () => ({ level: 1 }),
    },
    {
      tag: 'h3',
      getAttrs: () => ({ level: 2 }),
    },
    {
      tag: 'h4',
      getAttrs: () => ({ level: 3 }),
    },
    {
      tag: 'h5',
      getAttrs: () => ({ level: 4 }),
    },
    {
      tag: 'h6',
      getAttrs: () => ({ level: 5 }),
    },
  ],
  toDOM(node: Node) {
    const dom = document.createElement('stencila-heading')
    dom.draggable = true
    dom.id = node.attrs.id
    dom.setAttribute('level', node.attrs.level)

    const contentDOM = document.createElement(
      `h${Math.min(6, node.attrs.level + 1)}`
    )
    contentDOM.slot = 'content'
    dom.appendChild(contentDOM)

    if (node.attrs.authors) {
      const authors = document.createElement('div')
      authors.slot = 'authors'
      authors.innerHTML = node.attrs.authors
      dom.appendChild(authors)
    }

    return { dom, contentDOM }
  },
}

/**
 * A ProseMirror `NodeSpec` for a Stencila `IfBlock`
 */
const IfBlock: NodeSpec = {
  group: 'Block',
  content: 'IfBlockClause*',
  attrs: attrsWithDefault(null, ...executableAttrs),
  parseDOM: parseDOMWithContent(
    'stencila-if-block',
    '[slot=clauses]',
    ...executableAttrs
  ),
  toDOM: (node: Node) => {
    const dom = document.createElement('stencila-if-block')
    dom.draggable = true
    dom.id = node.attrs.id

    const contentDOM = document.createElement('div')
    contentDOM.slot = 'clauses'
    dom.appendChild(contentDOM)

    return { dom, contentDOM }
  },
}

/**
 * A ProseMirror `NodeSpec` for a Stencila `IfBlockClause`
 */
const IfBlockClause: NodeSpec = {
  group: 'Block',
  content: 'Block*',
  attrs: attrsWithDefault(null, ...executableAttrs),
  parseDOM: parseDOMWithContent(
    'stencila-if-block-clause',
    '[slot=content]',
    'is-active',
    ...codeExecutableAttrs
  ),
  toDOM: (node: Node) => {
    const dom = document.createElement('stencila-if-block-clause')
    dom.draggable = true
    dom.id = node.attrs.id

    const contentDOM = document.createElement('div')
    contentDOM.slot = 'content'
    dom.appendChild(contentDOM)

    return { dom, contentDOM }
  },
}

/**
 * A ProseMirror `NodeSpec` for a Stencila `InstructionBlock`
 */
const InstructionBlock: NodeSpec = {
  group: 'Block',
  content: 'Block*',
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

    const contentDOM = document.createElement('div')
    contentDOM.slot = 'content'
    dom.appendChild(contentDOM)

    if (node.attrs.messages) {
      const messages = document.createElement('div')
      messages.slot = 'messages'
      messages.innerHTML = node.attrs.messages
      dom.appendChild(messages)
    }

    if (node.attrs.authors) {
      const authors = document.createElement('div')
      authors.slot = 'authors'
      authors.innerHTML = node.attrs.authors
      dom.appendChild(authors)
    }

    if (node.attrs.suggestion) {
      const suggestion = document.createElement('div')
      suggestion.slot = 'suggestion'
      suggestion.innerHTML = node.attrs.suggestion
      dom.appendChild(suggestion)
    }

    return { dom, contentDOM }
  },
}

/**
 * A ProseMirror `NodeSpec` for a Stencila `List`
 */
const List: NodeSpec = {
  group: 'Block',
  content: 'ListItem*',
  attrs: {
    id: { default: null },
    order: { default: 'Unordered' },
    authors: { default: null },
  },
  parseDOM: [
    // For parsing Stencila DOM HTML
    {
      tag: 'stencila-list',
      contentElement: '[slot=items]',
      getAttrs: (elem: HTMLElement) => ({
        id: elem.getAttribute('id'),
        order: elem.getAttribute('order'),
        authors: elem.querySelector('[slot=authors]')?.innerHTML,
      }),
    },
    // For parsing any old HTML...
    {
      tag: 'ul',
      getAttrs: () => ({ order: 'Unordered' }),
    },
    {
      tag: 'ol',
      getAttrs: () => ({ order: 'Ascending' }),
    },
  ],
  toDOM: (node: Node) => {
    const dom = document.createElement('stencila-list')
    dom.draggable = true
    dom.id = node.attrs.id
    dom.setAttribute('order', node.attrs.order)

    const contentDOM = document.createElement(
      node.attrs.order === 'Ascending' ? 'ol' : 'ul'
    )
    contentDOM.slot = 'items'
    dom.appendChild(contentDOM)

    if (node.attrs.authors) {
      const authors = document.createElement('div')
      authors.slot = 'authors'
      authors.innerHTML = node.attrs.authors
      dom.appendChild(authors)
    }

    return { dom, contentDOM }
  },
}

/**
 * A ProseMirror `NodeSpec` for a Stencila `ListItem`
 */
const ListItem: NodeSpec = {
  content: 'Block*',
  attrs: attrsWithDefault(null, 'id'),
  parseDOM: [
    // For parsing Stencila DOM HTML
    {
      tag: 'stencila-list-item',
      contentElement: '[slot=content]',
      getAttrs: getAttrs('id'),
    },
    // For parsing any old HTML...
    {
      tag: 'li',
    },
  ],
  toDOM: (node: Node) => {
    const dom = document.createElement('stencila-list-item')
    dom.draggable = true
    dom.id = node.attrs.id

    const contentDOM = document.createElement('li')
    contentDOM.slot = 'content'
    dom.appendChild(contentDOM)

    return { dom, contentDOM }
  },
}

/**
 * A ProseMirror `NodeSpec` for a Stencila `Paragraph`
 */
const Paragraph: NodeSpec = {
  group: 'Block',
  content: 'Inline*',
  marks: '_',
  attrs: attrsWithDefault(null, 'id', 'authors'),
  parseDOM: [
    // For parsing Stencila DOM HTML
    {
      tag: 'stencila-paragraph',
      contentElement: '[slot=content]',
      getAttrs: (elem: HTMLElement) => ({
        id: elem.getAttribute('id'),
        authors: elem.querySelector('[slot=authors]')?.innerHTML,
      }),
    },
    // For parsing any old HTML...
    {
      tag: 'p',
    },
  ],
  toDOM(node: Node) {
    const dom = document.createElement('stencila-paragraph')
    dom.draggable = true
    dom.id = node.attrs.id

    const contentDOM = document.createElement('p')
    contentDOM.slot = 'content'
    dom.appendChild(contentDOM)

    if (node.attrs.authors) {
      const authors = document.createElement('div')
      authors.slot = 'authors'
      authors.innerHTML = node.attrs.authors
      dom.appendChild(authors)
    }

    return { dom, contentDOM }
  },
}

// Export specs and views

export const specs: Record<string, NodeSpec> = {
  Heading,
  IfBlock,
  IfBlockClause,
  InstructionBlock,
  List,
  ListItem,
  Paragraph,
}

export const views: Record<string, NodeViewConstructor> = {}
