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
      getAttrs: (elem: HTMLElement) => ({ level: 1, ...getAttrs('id')(elem) }),
    },
    {
      tag: 'h2',
      getAttrs: (elem: HTMLElement) => ({ level: 1, ...getAttrs('id')(elem) }),
    },
    {
      tag: 'h3',
      getAttrs: (elem: HTMLElement) => ({ level: 2, ...getAttrs('id')(elem) }),
    },
    {
      tag: 'h4',
      getAttrs: (elem: HTMLElement) => ({ level: 3, ...getAttrs('id')(elem) }),
    },
    {
      tag: 'h5',
      getAttrs: (elem: HTMLElement) => ({ level: 4, ...getAttrs('id')(elem) }),
    },
    {
      tag: 'h6',
      getAttrs: (elem: HTMLElement) => ({ level: 5, ...getAttrs('id')(elem) }),
    },
  ],
  toDOM(node: Node) {
    const dom = document.createElement('stencila-heading')
    dom.draggable = true
    dom.id = node.attrs.id

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
  Paragraph,
}

export const views: Record<string, NodeViewConstructor> = {}
