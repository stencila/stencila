import unified from 'unified'
import unistVisit from 'unist-util-visit'
import unistFind from 'unist-util-find'

import remarkParse from 'remark-parse'
import remarkSqueezeParagraphs from 'remark-squeeze-paragraphs'
import remarkBracketedSpans from 'remark-bracketed-spans'
import remarkSlug from 'remark-slug'
import remarkHtml from 'remark-html'
import remarkStringify from 'remark-stringify'

import rehypeParse from 'rehype-parse'
import rehype2remark from 'rehype-remark'
import rehypeStringify from 'rehype-stringify'


class DocumentMarkdownConverter {

  /**
   * Import from a Markdown document to a Stencila Document
   *
   * @param {string} md - Markdown content
   * @param {object} options - Conversion options
   * @return {object|string} - Document archive (virtual filesystem folder) or HTML string
   */
  import (md, options) {
    options = options || {}

    // Output options
    if (options.archive !== false) options.archive = true

    // Markdown parsing options
    if (options.gfm !== false) options.gfm = true
    if (options.commonmark !== false) options.commonmark = true
    options.fences = true

    // First-pass conversion to HTML
    const htmlInit = unified()
      // Parse Markdown into AST
      .use(remarkParse)
      // Modify Markdown AST using standard plugins
      .use(remarkSqueezeParagraphs)
      .use(remarkSlug)
      // Convert inputs with no default to MDAST type:'linkReference' nodes
      .use(remarkBracketedSpansEmptyTolinkReference)
      // Convert bracketed spans in MDAST type:'html' nodes with <span> elements
      .use(remarkBracketedSpansToHTML)
      // Use `remark-html` to create a HTML string. This deals with the {type: `html`} nodes
      // that `remarkBracketedSpansToHTML` introduces, whereas `remark-rehype` does not.
      // This seems to an inefficient way to do it because we then have to parse the HTML
      // But the alternative of tranforming the AST direcly MDAST -> HAST didn't work (escaped chars):
      //   .use(remark2rehype, {
      //     allowDangerousHTML: true
      //  })
      .use(remarkHtml)
      .processSync(md, options)

    // Second-pass tranformation of HTML
    const html = unified()
      .use(rehypeParse, {fragment: true})
      .use(rehypeCodeblockToCell)
      .use(rehypeSpanToInputOutput)
      .use(rehypeStringify)
      .processSync(htmlInit).contents.trim()

    if (options.archive) {
      return {
        'index.html': html
      }
    } else {
      return html
    }
  }

  /**
   * Export to a Markdown document from a Stencila Document
   *
   * @memberof document/markdown-converter
   *
   * @param {string|object} doc - Document archive (virtual filesystem folder) or HTML string
   * @param {object} options - Conversion options
   * @return {object|string} - Markdown content
   */
  export (doc, options) {
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
      .use(rehypeInputOutputToMarkdown)
      .use(rehypeBracketedSpansToMarkdown)
      .use(rehype2remark)
      .use(remarkStringify)
      .use(remarkBracketedSpansClean())
      .processSync(html, options).contents.trim()

    return md
  }
}

/**
 * Create `linkReference` for inputs with no default value
 * so they don't get ignored
 *
 * @returns {function} Transformer
 */
function remarkBracketedSpansEmptyTolinkReference () {
  return tree => {
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

// remark-bracketed-spans has several transformers, these function
// are just wrappers to provide a naming consistency

/**
 * Transform Pandoc style bracketed spans to MDAST type: 'html' nodes
 *
 * @returns {function} Transformer
 */
function remarkBracketedSpansToHTML () {
  return remarkBracketedSpans() // this is the md2html() function of remark-bracketed-spans
}

/**
 * Convert bracketed spans from MDAST to Markdown
 *
 * @returns {function} Transformer
 */
function rehypeBracketedSpansToMarkdown () {
  return remarkBracketedSpans.html2md()
}

/**
 * Cleans up Markdown created by `rehypeBracketedSpansToMarkdown`
 *
 * @returns {object} Visitors
 */
function remarkBracketedSpansClean () {
  return remarkBracketedSpans.mdVisitors
}

/**
 * Transform <span data-name=...> to <input> and <span data-value=...> to <output>
 *
 * @returns {function} Vistor function
 */
function rehypeSpanToInputOutput () {
  return tree => {
    unistVisit(tree, (node, i, parent) => { // eslint-disable-line no-unused-vars
      if (node.tagName === 'span' && node.properties) {
        if (node.properties.dataName) {
          // Set name attribute
          node.properties.name = node.properties.dataName // create 'name' attribute
          delete node.properties.dataName // remove 'data-name' attribute
          // Get value
          let value = node.children && node.children[0] && node.children[0].value // 'value' attribute is the content of the span

          if (node.properties.dataType === 'select') {
            node.tagName = 'select'
            delete node.properties.dataType
            node.children = []
            for (let property in node.properties) {
              if (property.substring(0, 4) === 'data') {
                let key = property.substring(4).toLowerCase()
                let label = node.properties[property]
                node.children.push({
                  type: 'element',
                  tagName: 'option',
                  properties: {
                    value: key,
                    selected: key === value ? 'true' : undefined
                  },
                  children: [{
                    type: 'text',
                    value: label
                  }]
                })
                delete node.properties[property]
              }
            }
          } else {
            node.tagName = 'input'

            // Primary attributes that inputs always or usually have
            if (value !== 'undefined') node.properties.value = value
            node.children = [] // inputs have no content

            // Secondary attributes : just rename all 'data-x' properties
            for (let property in node.properties) {
              if (property.substring(0, 4) === 'data') {
                node.properties[property.substring(4).toLowerCase()] = node.properties[property]
                delete node.properties[property]
              }
            }
          }
        } else if (node.properties.dataExpr) {
          node.tagName = 'output'
          // Set for attribute
          node.properties.for = node.properties.dataExpr // create 'for' attribute
          delete node.properties.dataExpr // remove 'data-expr' attribute
        }
      }
    })
  }
}

/**
 * Transform <span data-name=...> to <input> and <span data-value=...> to <output>
 *
 * @returns {function} Vistor function
 */
function rehypeInputOutputToMarkdown () {
  return tree => {
    unistVisit(tree, (node, i, parent) => { // eslint-disable-line no-unused-vars
      if ((node.tagName === 'input' || node.tagName === 'select') && node.properties && node.properties.name) {
        var name = node.properties.name
        delete node.properties.name

        if (node.tagName === 'input') {
          let value = node.properties.value
          if (!value) value = ''
          delete node.properties.value

          const type = node.properties.type
          delete node.properties.type

          const attr = Object.keys(node.properties).map(function (attrKey) {
            const attrValue = node.properties[attrKey].indexOf(' ') > -1
            ? `"${node.properties[attrKey]}"`
            : node.properties[attrKey]

            return `${attrKey}=${attrValue}`
          }).join(' ')

          node.type = 'text'
          node.value = `[${value}]{name=${name}${type ? ` type=${type}` : ''}${attr ? ` ${attr}` : ''}}`
        } else if (node.tagName === 'select') {
          let selected = ''
          const attr = node.children.map(child => {
            if (child.properties.selected) {
              selected = child.properties.value
            }
            const value = child.children[0].value.indexOf(' ') > -1
              ? `"${child.children[0].value}"`
              : child.children[0].value
            return `${child.properties.value}=${value}`
          }).join(' ')

          node.type = 'text'
          node.value = `[${selected}]{name=${name} type=select${attr ? ` ${attr}` : ''}}`
        }
      } else if (node.tagName === 'output') {
        const value = node.children && node.children[0] && node.children[0].value || ''
        const htmlFor = node.properties && node.properties.htmlFor

        delete node.properties.value
        delete node.properties.htmlFor

        const attr = Object.keys(node.properties).map(function (attrKey) {
          const attrValue = node.properties[attrKey]
          attrKey = attrKey.replace('data', '').toLowerCase()
          return `${attrKey}="${attrValue}"`
        }).join(' ')

        node.type = 'text'
        node.value = `[${value}]{expr=${htmlFor}${attr ? ` ${attr}` : ''}}`
      }
    })
  }
}

/**
 * Transform HTML codeblocks to Stencila cells
 *
 * Transforms HTML codeblocks i.e. `<pre><code class="language-...">` into Stencila cells
 * i.e. `<div data-cell="...">`
 *
 * @return {function} Transformer
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

            // Change this codeblock <pre><code class="language-..."> into cell <div data-cell="...">
            parent.tagName = 'div'
            parent.properties['data-cell'] = expr
            parent.children = children
          }
        }
      }
    })
  }
}

/**
 * Transform Stencila cells to HTML codeblocks
 *
 * Transforms Stencila cells i.e. `<div data-cell="...">` into HTML codeblocks
 * i.e. `<pre><code class="language-...">`
 *
 * @return {function} Transformer
 */
function rehypeCellToCodeblock () {
  return tree => {
    unistVisit(tree, (node, i, parent) => { // eslint-disable-line no-unused-vars
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
              children: [{
                type: 'text',
                value: expr
              }]
            }
          }
          // Set the 'class' of <code>
          code.properties.className = [className]

          // Change this cell <div> into a codeblock <pre><code>
          node.tagName = 'pre'
          node.properties = {}
          node.children = [code]
        }
      }
    })
  }
}

