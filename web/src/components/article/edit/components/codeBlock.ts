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
      programmingLanguage: { default: '' },
    },
    toDOM(node) {
      return [
        'stencila-editor',
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
      {
        tag: 'stencila-editor',
        preserveWhitespace: 'full',
        contentElement: '.cm-content',
        getAttrs(dom) {
          const elem = dom as HTMLStencilaEditorElement
          return {
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
