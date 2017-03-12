import unified from 'unified'

import remarkParse from 'remark-parse'
import remark2rehype from 'remark-rehype'
import remarkSlug from 'remark-slug'
import remarkStringify from 'remark-stringify'

import rehypeParse from 'rehype-parse'
import rehype2remark from 'rehype-remark'
import rehypeStringify from 'rehype-stringify'

export default {

  import: function (md, options) {
    options = options || {}

    // Output options
    if (options.archive !== false) options.archive = true

    // Markdown parsing options
    if (options.gfm !== false) options.gfm = true
    if (options.commonmark !== false) options.commonmark = true
    options.fences = true

    const html = unified()
      .use(remarkParse)
      .use(remarkSlug)
      .use(remark2rehype)
      .use(rehypeStringify)
      .processSync(md, options).contents.trim()

    if (options.archive) {
      return {
        'index.html': html
      }
    } else {
      return html
    }
  },

  export: function (doc, options) {
    options = options || {}

    let html = typeof doc === 'string' ? doc : doc['index.html']

    // See the `remark-stringify` options at https://github.com/wooorm/remark/tree/master/packages/remark-stringify#options
    if (options.gfm !== false) options.gfm = true
    // If commonmark == true remark collapses adjacent blockquotes
    // This is confusing because the remark README says that it will "Compile adjacent blockquotes separately"
    if (!options.commonmark) options.commonmark = false
    if (options.fragment !== false) options.fragment = true
    options.listItemIndent = '1'
    options.strong = '*'
    options.emphasis = '_'
    options.fences = true
    options.rule = '-'
    options.ruleRepetition = 3
    options.ruleSpaces = false
    options.entities = false
    options.encode = false

    const md = unified()
      .use(rehypeParse)
      .use(rehype2remark)
      .use(remarkStringify)
      .processSync(html, options).contents.trim()

    return md
  }
}
