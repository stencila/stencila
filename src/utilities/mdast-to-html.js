import unified from 'unified'
import remarkParse from 'remark-parse'
import remarkStringify from 'remark-stringify'
import remarkHtml from 'remark-html'
import squeezeParagraphs from 'remark-squeeze-paragraphs'
import slug from 'remark-slug'

export default function mdast2html (mdast, options) {
  options = options || {}
  if (options.gfm !== false) options.gfm = true
  if (options.commonmark !== false) options.commonmark = true
  options.fences = true
  options.fragment = true

  const html = unified()
    .use(remarkParse)
    .use(squeezeParagraphs)
    .use(slug)
    // include directive not available here
    // .use(include.md2html)
    .use(remarkStringify)
    .use(remarkHtml)

  return html.stringify(mdast, options)
}
