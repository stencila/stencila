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

import {DefaultDOMElement} from 'substance'

import {
  LIST as languageLIST,
  shortname as languageShortname, 
  comment as languageComment
} from '../language'


export default class DocumentMarkdownConverter {

  /**
   * Is a file name to be converted using this converter? 
   * 
   * @param  {string} path - Name of file
   * @return {boolean} - Is the file name matched?
   */
  static match (path, storer) {
    return path.slice(-3) === '.md'
  }

  /**
   * Convert a codeblock shebang line to a Mini expression
   * 
   * @param  {String} line     Line (usually first) of codeblock to covert
   * @param  {String} language Name of language for codeblock
   * @return {null|String}     Mini expression, or null if not a shebang line
   */
  static shebangToMini(line, lang = '') {
    lang = languageShortname(lang)
    let comment = languageComment(lang)
    let match = line.match(`^${comment}!\\s*((\\w+)\\s*=\\s*)?(function|global)?\\s*(.*)`)
    if (!match) return null
    let mini = ''
    if (match[2]) mini = match[2] + ' = '
    if (match[3]) mini += match[3] + ' '
    mini += lang
    if (match[4]) mini += match[4]
    else mini += '()'
    return mini
  }

  /**
   * Convert a Mini expression to a language code and a shebang line
   * 
   * @param  {String} mini     Mini expression
   * @return {Object}          {lang, shebang}  
   */
  static miniToShebang(mini) {
    let match = mini.match(`^\\s*((\\w+)\\s*=\\s*)?(function|global)?\\s*(${languageLIST.join('|')})+\\s*(.*)`)
    if (!match) {
      return {
        lang: 'mini',
        shebang: null
      }
    }

    let lang = languageShortname(match[4])
    let comment = languageComment(lang)
    let shebang = comment + '!'
    if (match[2]) shebang += ' ' + match[2] + ' ='
    if (match[3]) shebang += ' ' + match[3]
    if (match[5] && match[5] !== '()') shebang += ' ' + match[5]
    return {
      lang: lang,
      shebang: shebang
    }
  }

  import(path, storer, buffer) {
    return storer.readFile(path).then(md => {
      let html = `
        <!DOCTYPE html>
        <html>
          <head>
            <title></title>
          </head>
          <body>
            <main>
              <div id="data" data-format="html">
                <div class="content">${this.importContent(md)}</div>
              </div>
            </main>
          </body>
        </html>
      `
      return buffer.writeFile('index.html', html).then(() => {
        return html
      })
    })
  }

  export(path, storer, buffer) {
    return buffer.readFile('index.html').then((html) => {
      let content = DefaultDOMElement.parseHTML(html).find('.content')
      if (!content) throw new Error('No div.content element in HTML!')
      let md = this.exportContent(content.getInnerHTML())
      return storer.writeFile(path, md)
    })
  }

  /**
   * Convert a Markdown document to a Stencila Document as HTML
   *
   * @param {string} md - Markdown content
   * @param {object} options - Conversion options
   * @return {string} - HTML string
   */
  importContent (md, options) {
    options = options || {}

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

    return html
  }

  /**
   * Export Markdown content from a Stencila Document as HTML
   *
   * @param {string|object} html - HTML string
   * @param {object} options - Conversion options
   * @return {object|string} - Markdown content
   */
  exportContent (html, options) {
    options = options || {}

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

    let md = unified()
      .use(rehypeParse)
      .use(rehypeCellToCodeblock)
      .use(rehypeInputOutputToMarkdown)
      .use(rehypeBracketedSpansToMarkdown)
      .use(rehype2remark)
      .use(remarkStringify)
      .use(remarkBracketedSpansClean())
      .processSync(html, options).contents.trim()

    // Not clear why this is necessary and whether it is better
    // dealt with by options to rehype/remark
    md = md.replace(/&lt;/g, '<')

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
 * Transform <span data-name=...> to <input> and <span data-expr=...> to <output>
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
      if (parent && node.tagName === 'code') {
        if (parent.tagName === 'pre') {
          // Rehype adds a trailing newline to the code, remove it
          let code = ''
          if (node.children.length) {
            code = node.children[0].value
            if (code.slice(-1) === '\n') {
              code = code.slice(0, -1)
            }
            node.children[0].value = code
          }
          if (node.properties.className && node.properties.className.length) {
            let className = node.properties.className[0]
            let match = className.match(/^language-([^ ]+)$/)
            if (match) {
              let language = match[1]

              let mini
              let children
              if (language === 'mini') {
                mini = code
                children = []
              } else {
                // Convert first line to Mini
                let lines = code.split('\n')
                mini = DocumentMarkdownConverter.shebangToMini(lines[0], language)
                // If the first line was a shebang then strip it
                if (mini) {
                  code = lines.slice(1).join('\n')
                  // Create <pre> for code
                  children = [{
                    type: 'element',
                    tagName: 'pre',
                    properties: {
                      'data-source': ''
                    },
                    children: [{
                      type: 'text',
                      value: code
                    }]
                  }]
                }
              }

              // Not mini, or no shebang, don't change this codeblock to a cell
              if (!mini) return

              // Change this codeblock <pre><code class="language-..."> into 
              // cell <div data-cell="..."><pre>
              parent.tagName = 'div'
              parent.properties['data-cell'] = mini
              parent.children = children
            }
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
        let mini = node.properties.dataCell // 'data-cell' is parsed to 'dataCell'
        if (mini) {
          // Get code
          let code
          let pre = unistFind(node, {tagName: 'pre'})
          if (pre) code = pre.children[0].value
          else code = mini

          // Get language
          let {lang, shebang} = DocumentMarkdownConverter.miniToShebang(mini)
          let className = `language-${lang}`
          if (shebang) {
            code = shebang + '\n' + code
          }

          // Change this cell <div> into a codeblock <pre><code>
          node.tagName = 'pre'
          node.properties = {}
          node.children = [{
            type: 'element',
            tagName: 'code',
            properties: {
              className: [className]
            },
            children: [{
              type: 'text',
              value: code
            }]
          }]
        }
      }
    })
  }
}

