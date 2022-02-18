import { NodeSpec } from 'prosemirror-model'

/**
 * Generate a `NodeSpec` to represent a Stencila `CodeChunk`
 *
 * Based on https://prosemirror.net/examples/codemirror/ and https://github.com/ProseMirror/prosemirror-schema-basic/blob/b5ae707ab1be98a1d8735dfdc7d1845bcd126f18/src/schema-basic.js#L59
 */
export function codeChunk(): NodeSpec {
  return {
    allowGapCursor: true,
    atom: true,
    code: true,
    content: 'text*',
    defining: true,
    draggable: true,
    inline: false,
    isolating: false,
    marks: '',
    selectable: true,
    group: 'BlockContent',
    attrs: {
      id: { default: '' },
      itemtype: { default: '' },
      programmingLanguage: { default: '' },
    },
    toDOM(node) {
      return [
        'stencila-code-chunk',
        {
          'active-language':
            typeof node.attrs.programmingLanguage === 'string'
              ? node.attrs.programmingLanguage
              : undefined,
        },
        [
          'pre',
          {
            slot: 'text',
          },
          0,
        ],
      ]
    },
    parseDOM: [
      {
        tag: '[itemtype="https://schema.stenci.la/CodeChunk"]',
        preserveWhitespace: 'full',
        contentElement: 'code',
        getAttrs(dom) {
          const elem = dom as HTMLElement
          return {
            id: elem.getAttribute('id'),
            itemtype: elem.getAttribute('itemtype'),
            programmingLanguage:
              elem
                .querySelector('meta[itemprop="programmingLanguage"][content]')
                ?.getAttribute('content') ??
              elem
                .querySelector('code[class^="language-"]')
                ?.getAttribute('class')
                ?.substring(9),
          }
        },
      },
      {
        tag: 'stencila-code-chunk',
        preserveWhitespace: 'full',
        contentElement: '.cm-content',
        getAttrs(dom) {
          const elem = dom as HTMLStencilaCodeChunkElement
          return {
            id: elem.getAttribute('id'),
            itemtype: elem.getAttribute('itemtype'),
            programmingLanguage:
              elem.querySelector<HTMLSelectElement>(
                '[aria-label="Programming Language"] select'
              )?.value ?? elem.getAttribute('active-language'),
          }
        },
      },
    ],
  }
}
