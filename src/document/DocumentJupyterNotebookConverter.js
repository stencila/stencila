const cheerio = require('cheerio')

const context = require('../context/context')
const markdown = require('../utilities/markdown')

/**
 * Jupyter notebook coonverter for the `Document` class
 *
 * Converts a document from/to a Jupyter notebook based on
 * the documentation of the notebook format at https://github.com/jupyter/nbformat.
 * See there for JSON schemas too e.g. https://github.com/jupyter/nbformat/blob/master/nbformat/v4/nbformat.v4.schema.json
 */
class DocumentJupyterNotebookConverter {

  /**
   * Load a document from a Jupyter notebook
   *
   * @param  {Document} doc - Document to load
   * @param  {String} content - Notebook content
   * @param  {Object} options - Any options (see implementations for those available)
   */
  load (doc, content, options) {
    // Get notebook data, by parsing JSON string if necessary
    let data = (typeof content === 'string') ? JSON.parse(content) : content

    // Create the DOM
    let dom = cheerio.load('')
    let $root = dom.root()

    // Get metadata
    let metadata = data.metadata

    // Get language
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
    let langCode = context.language.code(lang) || ''

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
      if (cell.cell_type === 'markdown') {
        let html = markdown.md2html(source)
        $root.append(html)
      } else if (cell.cell_type === 'code') {
        let $pre = cheerio('<pre>')
                    .attr('data-execute', langCode)
                    .text(source)
        $root.append($pre)

        let outputs = cell.outputs
        if (outputs) {
          for (let output of outputs) {
            let type = output.output_type
            if (type === 'execute_result' || type === 'display_data') {
              let mimebundle = output.data
              let type = Object.keys(mimebundle)[0]
              let value = mimebundle[type]
              let $el
              if (type === 'image/png') {
                $el = cheerio('<img>').attr('src', `data:${type};base64,${value}`)
              } else {
                $el = cheerio('<pre>').text(value)
              }
              // TODO : deal with other mime types
              // For examples see https://raw.githubusercontent.com/SciRuby/sciruby-notebooks/master/getting_started.ipynb
              $el.attr('data-output', true)
              $root.append($el)
            } else if (type === 'stream') {
              let $el
              if (output.name === 'stderr') {
                $el = cheerio('<pre>').attr('data-errors', true)
              } else {
                $el = cheerio('<pre>').attr('data-output', true)
              }
              $el.text(output.text.join(''))
              $root.append($el)
            } else if (type === 'error') {
              let $el = cheerio('<pre>').attr('data-errors', true)
              $el.text(output.ename + ': ' + output.evalue + '\n\n' + output.traceback)
              $root.append($el)
            }
          }
        }
      }
    }

    doc.content = dom
  }

  /**
   * Dump a document to a Jupyter notebook
   *
   * @param  {Document} doc - Document to dump
   * @param  {Object} options - Any options (see implementations for those available)
   * @returns {String} - Content of the document as HTML
   */
  dump (doc, options) {
    options = options || {}
    if (options.stringify !== false) options.stringify = true
    if (options.pretty !== false) options.pretty = true

    let nb = {
      cells: [],
      metadata: {}, // TODO lang from executes - ensure only one
      nbformat: 4,
      nbformat_minor: 2
    }

    // We need to accumulate HTML elements which are then flushed
    // into a markdown cell when an execute directive is hit, or at end
    let html = ''
    function flush () {
      if (html) {
        let md = markdown.html2md(html)
        let lines = md.split('\n').map(line => `${line}\n`)
        nb.cells.push({
          cell_type: 'markdown',
          metadata: {},
          source: lines
        })
      }
      html = ''
    }

    // Iterate over elements
    doc.content.root().children().each(function () {
      let $this = cheerio(this)
      if ($this.is('[data-execute]')) {
        flush()
        let source = $this.text()
        let lines = source.split('\n').map(line => `${line}\n`)
        nb.cells.push({
          cell_type: 'code',
          metadata: {}, // TODO
          source: lines,
          outputs: [], // TODO
          execution_count: null // TODO
        })
      } else {
        html += cheerio.html($this)
      }
    })
    flush()

    if (options.stringify) {
      return JSON.stringify(nb, null, options.pretty ? '  ' : null)
    } else {
      return nb
    }
  }
}

module.exports = DocumentJupyterNotebookConverter
