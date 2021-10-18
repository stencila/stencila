import { NodeSpec } from 'prosemirror-model'

/**
 * Generate a `NodeSpec` to represent a Stencila `CodeBlock`
 *
 * This is temporary and will be replaced with a CodeMirror editor
 * (see https://prosemirror.net/examples/codemirror/ and https://gist.github.com/BrianHung/08146f89ea903f893946963570263040).
 *
 * Based on https://github.com/ProseMirror/prosemirror-schema-basic/blob/b5ae707ab1be98a1d8735dfdc7d1845bcd126f18/src/schema-basic.js#L59
 */
export function codeBlock(): NodeSpec {
  return {
    content: 'text*',
    marks: '',
    code: true,
    atom: false,
    allowGapCursor: true,
    isolating: true,
    selectable: true,
    group: 'BlockContent',
    inline: false,
    defining: true,
    draggable: true,
    attrs: {
      programmingLanguage: { default: '' },
    },
    toDOM(node) {
      const textContent: string[] = []

      node.content.forEach((n) => {
        if (n.text) {
          textContent.push(n.text)
        }
      })

      return [
        'stencila-editor',
        {
          'active-language': node.attrs.programmingLanguage,
        },
        [
          'pre',
          {
            slot: 'text',
          },
          ['code', { spellcheck: 'false' }, textContent.join('/n')],
        ],
      ]
    },
    parseDOM: [
      {
        tag: '[itemtype="http://schema.stenci.la/CodeBlock"]',
        preserveWhitespace: 'full',
        contentElement: 'code',
        getAttrs(dom) {
          const elem = dom as HTMLElement
          return {
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
    ],
  }
}
