import { NodeSpec } from 'prosemirror-model'

export const article: Record<string, NodeSpec> = {
  Article: {
    group: 'CreativeWork',
    content: 'ArticleTitle? ArticleDescription? ArticleContent',
    parseDOM: [{ tag: 'article' }],
    toDOM: () => ['article', 0],
  },
  ArticleTitle: {
    content: 'InlineContent*',
    parseDOM: [{ tag: '[data-prop=title]' }],
    toDOM: () => ['div', { 'data-prop': 'title' }, 0],
  },
  ArticleDescription: {
    content: 'BlockContent*',
    parseDOM: [{ tag: '[data-prop=description]' }],
    toDOM: () => ['div', { 'data-prop': 'description' }, 0],
  },
  ArticleContent: {
    content: 'BlockContent*',
    parseDOM: [{ tag: '[data-prop=content]' }],
    toDOM: () => ['div', { 'data-prop': 'content' }, 0],
  },
}
