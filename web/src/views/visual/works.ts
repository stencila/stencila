import { NodeSpec, parseToDOM } from './prelude'

/**
 * A ProseMirror `NodeSpec` for a Stencila `Article`
 *
 * This may get split up into separate sections as in v1 or similar.
 * https://github.com/stencila/stencila/blob/v1/web/src/components/editors/prose-editor/nodes/article.ts
 */
export const Article: NodeSpec = {
  group: 'CreativeWorkType',
  content: 'Block*',
  attrs: {
    id: {},
  },
  ...parseToDOM('stencila-article', 'id'),
}

export const works = { Article }
