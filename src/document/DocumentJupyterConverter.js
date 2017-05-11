// Use Substance's DOM implementation (works in browser and in node). See
//    https://github.com/substance/substance/blob/develop/dom/DOMElement.js
// for the API
import {DefaultDOMElement} from 'substance'

import htmlEntities from 'html-entities'
const he = htmlEntities.Html5Entities

// Standard language codes
import * as language from '../language'

// Markdown <-> HTML conversion for Jupyter Markdown cells
import MarkdownConverter from '../document/DocumentMarkdownConverter'
const markdownConverter = new MarkdownConverter()
const md2html = md => markdownConverter.importContent(md)
const html2md = html => markdownConverter.exportContent(html)

// Cell value <-> HTML conversion for Jupyter code cell outputs
import {fromHTML, toHTML, fromMime, toMime} from '../value'

/**
 * Jupyter notebook converter for the Stencila Documents
 *
 * Converts a document from/to a Jupyter Notebook based on
 * the documentation of the notebook format at
 *
 *   https://github.com/jupyter/nbformat
 *
 * See there for JSON schemas too
 *
 *   e.g. https://github.com/jupyter/nbformat/blob/master/nbformat/v4/nbformat.v4.schema.json
 *
 * @namespace document/jupyter
 */
export default {

  /**
   * Convert from a Jupyter Notebook to a Stencila Document
   *
   * @memberof document/jupyter
   *
   * @param  {string} content - Content of Jupyter Notebook (JSON)
   * @param {object} options - Conversion options
   * @return {string} Content of Stencila Document (HTML)
   */
  import: function (content, options) {
    options = options || {}
    if (options.archive !== false) options.archive = true

    // Get notebook data, by parsing JSON string if necessary
    let data = (typeof content === 'string') ? JSON.parse(content) : content

    let doc = DefaultDOMElement.createDocument('html')
    let $$ = doc.createElement.bind(doc)
    let root = $$('div')

    // Get notebook metadata
    let metadata = data.metadata

    // Get notebook language
    let lang
    if (metadata) {
      if (metadata.language_info) {
        lang = metadata.language_info.name
      } else if (metadata.kernelspec) {
        lang = metadata.kernelspec.language
      } else if (metadata.kernel_info) {
        lang = metadata.kernel_info.language
      }
    }
    lang = language.shortname(lang || '') || ''

    // Get cells
    let cells
    if (data.cells) {
      cells = data.cells
    } else if (data.worksheets) {
      // In nbformat 3.0 there is an array called worksheets, each having cells
      cells = data.worksheets[0].cells
    }

    // Convert each cell
    for (let cell of cells) {
      let source = cell.source.join('')

      // Remove the trailing newline, otherwise we get an extra line when dumping
      if (source.slice(-1) === '\n') source = source.slice(0, -1)

      if (cell.cell_type === 'markdown') {
        // Convert Markdown to HTML and insert into the document
        let html = md2html(source)
        root.append(html)
      } else if (cell.cell_type === 'code') {
        // Create a new chunk (ie. a cell with a `run` mini expression)
        let chunk = $$('div').attr('data-cell', 'run')

        // Append source code escaped for &, <, >, ", ', and `
        source = he.encode(source)
        chunk.append(
          $$('pre').attr('data-source', lang).text(source)
        )

        // Process outputs
        let outputs = cell.outputs
        if (outputs) {
          for (let output of outputs) {
            let type = output.output_type
            if (type === 'execute_result' || type === 'display_data') {
              // Get output mimetype and content
              // Currently just use the first mime type (there may be multiple
              // formats for the output)
              let mimebundle = output.data
              let mimetype = Object.keys(mimebundle)[0]
              let content = mimebundle[mimetype]
              if (content.constructor.name === 'Array') content = content.join('')

              // Convert MIME type/content to HTML and append to chunk
              let value = fromMime(mimetype, content)
              let html = toHTML(value)
              // Use a temporary wrapper div to load html
              let el = $$('div').html(html).children[0]
              chunk.append(el)
            } else if (type === 'stream') {
              let out = $$('pre')
              if (output.name === 'stderr') out.attr('data-error', '')
              else {
                out.attr('data-value', 'str')
                out.attr('data-format', 'text')
              }
              out.text(output.text.join(''))
              chunk.append(out)
            } else if (type === 'error') {
              let error = $$('pre').attr('data-error', '')
              error.text(output.ename + ': ' + output.evalue + '\n\n' + output.traceback)
              chunk.append(error)
            }
          }
        }

        root.append(chunk)
      }
    }

    let html = root.html()
    if (options.archive) {
      return {
        'index.html': html
      }
    } else {
      return html
    }
  },

  /**
   * Convert to a Jupyter Notebook from a Stencila Document
   *
   * @memberof document/jupyter
   *
   * @param {string|object} doc - Document archive (virtual filesystem folder) or HTML string
   * @param {object} options - Conversion options
   * @return {object|string} - Jupyter notebook
   */
  export: function (doc, options) {
    options = options || {}
    if (options.stringify !== false) options.stringify = true
    if (options.pretty !== false) options.pretty = true
    if (options.archive !== false) options.archive = true

    // Initial, empty notebook object
    let nb = {
      cells: [],
      metadata: {}, // TODO lang from executes - ensure only one
      nbformat: 4,
      nbformat_minor: 2
    }

    // Create a DOM document to iterate over
    let html = typeof doc === 'string' ? doc : doc['index.html']
    let root = DefaultDOMElement.createDocument('html').setInnerHTML(html)

    // We need to accumulate HTML elements which are then flushed
    // into a markdown cell when a Stencila cell is hit, or at end
    let buffer = ''
    function flush () {
      if (buffer) {
        let md = html2md(buffer)
        let lines = md.split('\n').map(line => `${line}\n`)
        nb.cells.push({
          cell_type: 'markdown',
          metadata: {},
          source: lines
        })
      }
      buffer = ''
    }

    root.children.forEach(child => {
      if (child.is('[data-cell]')) {
        // Flush any HTML into a Markdown cell
        flush()

        // Create source lines
        let source = child.find(['[data-source]']).text()
        source = he.decode(source)
        let lines = source.split('\n').map(line => `${line}\n`)

        // Create outputs
        let outputs = []
        child.findAll(['[data-value]']).forEach(elem => {
          let value = fromHTML(elem)
          let mime = toMime(value)
          let mimebundle = {}
          mimebundle[mime.mimetype] = mime.content
          let output = {
            output_type: 'execute_result',
            data: mimebundle
          }
          outputs.push(output)
        })

        nb.cells.push({
          cell_type: 'code',
          metadata: {}, // TODO
          source: lines,
          outputs: outputs,
          execution_count: null // TODO
        })
      } else {
        // Accumulate HTML buffer
        buffer += child.outerHTML
      }
    })
    // Flush any remaining HTML into a Markdown cell
    flush()

    let content
    if (options.stringify) {
      content = JSON.stringify(nb, null, options.pretty ? '  ' : null)
    } else {
      content = nb
    }

    if (options.archive) {
      return {
        '.': content
      }
    } else {
      return content
    }
  }

}
