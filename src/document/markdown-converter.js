import unified from 'unified'
import unistVisit from 'unist-util-visit'
import unistFind from 'unist-util-find'
import unistRemove from 'unist-util-remove'

import remarkParse from 'remark-parse'
import remarkSqueezeParagraphs from 'remark-squeeze-paragraphs'
import remarkBracketedSpans from 'remark-bracketed-spans'
import remarkSlug from 'remark-slug'
import remark2rehype from 'remark-rehype'
import remarkStringify from 'remark-stringify'

import rehypeParse from 'rehype-parse'
import rehype2remark from 'rehype-remark'
import rehypeStringify from 'rehype-stringify'

/**
 * @namespace document/markdown-converter
 */
export default {

  /**
   * Import from a Markdown document to a Stencila Document
   *
   * @memberof document/markdown-converter
   *
   * @param {string} md - Markdown content
   * @param {object} options - Conversion options
   * @return {object|string} - Document archive (virtual filesystem folder) or HTML string
   */
  import: function (md, options) {
    options = options || {}

    // Output options
    if (options.archive !== false) options.archive = true

    // Markdown parsing options
    if (options.gfm !== false) options.gfm = true
    if (options.commonmark !== false) options.commonmark = true
    options.fences = true

    const html = unified()
      // Parse Markdown into AST
      .use(remarkParse)
      // Modify Markdown AST
      //.use(look)
      .use(remarkSqueezeParagraphs)
      .use(remarkSlug)
      // Transformations for inputs and outputs (inline cells) which use Markdown bracketed spans
      // Not clear on what is happening here, just refactored @sethvincent's orginal code
      //.use(remarkBracketedSpansEmpty)
      //.use(remarkBracketedSpans)
      //.use(remarkStringify)
      //.use(inputsOutputsMarkdownToHTML)
      //.use(remarkHtml)
      .use(remark2rehype)
      .use(rehypeCodeblockToCell)
      .use(rehypeStringify)
      // Run it all
      .processSync(md, options).contents.trim()

    if (options.archive) {
      return {
        'index.html': html
      }
    } else {
      return html
    }
  },

  /**
   * Export to a Markdown document from a Stencila Document
   *
   * @memberof document/markdown-converter
   *
   * @param {string|object} doc - Document archive (virtual filesystem folder) or HTML string
   * @param {object} options - Conversion options
   * @return {object|string} - Markdown content
   */
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
      .use(rehypeCellToCodeblock)
      .use(rehype2remark)
      .use(remarkStringify)
      .processSync(html, options).contents.trim()

    return md
  }
}

function look () {
  return tree => {
    unistVisit(tree, 'paragraph', (node, i, parent) => {
      console.log(node)
    })
  }
}



/**
 * Create `linkReference` for inputs with no default value
 *
 * @returns {function} Vistor function
 */
function remarkEmptyBracketedSpans () {
  return function (tree) {
    unistVisit(tree, function (node, i, parent) {
      if (node.value && node.value.indexOf('[]') > -1) {
        var value = node.value.split('[]')
        node.value = value[0]

        parent.children.splice(i + 1, 0, {
          type: 'linkReference',
          referenceType: 'shortcut',
          children: []
        })

        parent.children.splice(i + 2, 0, {
          type: 'text',
          value: value[1]
        })
      }
    })
  }
}

/**
 * Converts input and output Markdown to HTML
 *
 * @returns {function} Vistor function
 */
function inputsOutputsMarkdownToHTML () {
  return function (tree) {
    unistVisit(tree, function (node, i, parent) {
      const data = remarkBracketedSpans.parseMarkdown(node, i, parent, tree)
      if (!data) return
      let trailingText
      let html

      if (data.attr.name) {
        const value = data.text || ''
        const attr = data.attr
        const inputType = attr.type
        const name = attr.name

        delete attr.name
        delete attr.type

        if (inputType === 'select') {
          html = `<select name="${name}">${Object.keys(attr).map((key) => {
            return `<option value="${key}"${value === key ? ` selected="true"` : ''}>${attr[key]}</option>`
          }).join('')}</select>`
        } else {
          const htmlattr = Object.keys(attr).map((key) => {
            return `${key}="${attr[key]}"`
          }).join(' ')

          html = `<input name="${name}"${inputType ? ` type="${inputType}"` : ''}${htmlattr ? ` ${htmlattr}` : ''} value="${value}">`
        }
      } else if (data.attr.value) {
        const forInput = data.attr.value
        delete data.attr.value

        const htmlattr = Object.keys(data.attr).map((key) => {
          return `data-${key}="${data.attr[key]}"`
        }).join(' ')

        html = `<output for="${forInput}"${htmlattr ? ` ${htmlattr}` : ''}>${data.text ? data.text : ''}</option>`
      } else {
        html = data.html
        trailingText = data.trailingText
      }

      parent.children[i] = {
        type: 'html',
        value: html
      }

      if (data.trailingText) {
        parent.children[i + 1] = {
          type: 'text',
          value: trailingText
        }
      } else {
        unistRemove(parent, parent.children[i + 1])
      }
    })
  }
}

/**
 * Converts HTML codeblocks to HTML Stencila cells
 *
 * @return {function} Tranformer
 */
function rehypeCodeblockToCell () {
  return tree => {
    unistVisit(tree, (node, i, parent) => {
      if (parent && node.tagName === 'code' && node.properties.className) {
        if (parent.tagName === 'pre' && node.properties.className.length) {
          let className = node.properties.className[0]
          let match = className.match(/(language-(\.))|(run{([\w-]+)})|((\w+=)?call(\([^)]+\))?{([\w-]+)})/)
          if (match) {
            let mini = match[1]
            let run = match[3]
            let call = match[5]
            let expr
            let children
            if (mini) {
              expr = node.children[0].value
            } else if (run || call) {
              let language = run ? match[4] : match[8]
              node.properties = {
                className: [`language-${language}`],
                'data-source': language
              }
              expr = run ? 'run' : 'call'

              children = [{
                type: 'element',
                tagName: 'pre',
                children: [node]
              }]
            }

            parent.tagName = 'div'
            parent.properties['data-cell'] = expr
            parent.children = children
          }
        }
      }
    })
  }
}

function rehypeCellToCodeblock () {
  return tree => {
    unistVisit(tree, (node, i, parent) => {
      if (node.type === 'element' && node.properties) {
        let expr = node.properties.dataCell // 'data-cell' is parsed to 'dataCell'
        if (expr) {
          // Get (for `run` and `call` cells) or create (for `mini` cells)
          // a code element
          let className
          let code = unistFind(node, {tagName: 'code'})
          if (code) {
            // Must be a `run` or `call`
            let lang = code.properties.dataSource
            className = `language-${expr}{${lang}}`
          } else {
            // Must be a mini cell
            className = 'language-.'
            code = {
              type: 'element',
              tagName: 'code',
              properties: {},
              children: {
                type: 'text',
                value: expr
              }
            }
          }
          code.properties.className = [className]

          node.tagName = 'pre'
          node.properties = {}
          node.children = [code]
        }
      }
    })
  }
}

