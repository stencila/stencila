const cheerio = require('cheerio')

const context = require('../context/context')
const markdown = require('../utilities/markdown')

/**
 * Jupyter notebook coonverter for the `Document` class
 *
 * Converts a document from/to a Jupyter notebook based on
 * the documentation of the notebook format at https://github.com/jupyter/nbformat.
 * See there for JSOn schemas too e.g. https://github.com/jupyter/nbformat/blob/master/nbformat/v4/nbformat.v4.schema.json
 */
class DocumentJupyterNotebookConverter {

  /**
   * Load a document from a Jupyter notebook
   *
   * - `nbformat` and `nbformat_minor` are curretly ignored
   * - `metadata.kernel_info` and `metadata.language_info` are used to determine the
   *    language to be assumed for code cells when they are converted to execute directives
   * - Markdown cells are converted to HTML and
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
      let source = cell.source.join('\n')
      if (cell.cell_type === 'markdown') {
        let html = markdown.md2html(source)
        $root.append(html)
      } else if (cell.cell_type === 'code') {
        let $pre = cheerio('<pre>')
                    .attr('data-execute', langCode)
                    .text(source)
        $root.append($pre)
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
    let cells = []

    // We need to accumulate HTML elements which are then flushed
    // into a markdown cell when an execute directive is hit, or at end
    let html = ''
    function flush () {
      if (html) {
        let md = markdown.html2md(html)
        cells.push({
          cell_type: 'markdown',
          metadata: {},
          source: md.split('\n')
        })
      }
      html = ''
    }

    // Iterate over elements
    doc.content.root().children().each(function () {
      let $this = cheerio(this)
      if ($this.is('[data-execute]')) {
        flush()
        cells.push({
          cell_type: 'code',
          metadata: {}, // TODO
          source: $this.text().split('\n'),
          outputs: [], // TODO
          execution_count: null // TODO
        })
      } else {
        html += cheerio.html($this)
      }
    })
    flush()

    // Notebooks are usually pretty printed so follow suit
    return JSON.stringify({
      cells: cells,
      metadata: {}, // TODO
      nbformat: 4,
      nbformat_minor: 2
    }, null, '  ')
  }
}

module.exports = DocumentJupyterNotebookConverter
