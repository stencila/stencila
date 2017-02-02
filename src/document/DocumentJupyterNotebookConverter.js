const cheerio = require('cheerio')

const markdown = require('../utilities/markdown')

/**
 * Jupyter notebook coonverter for the `Document` class
 *
 * Converts a document from/to a Jupyter notebook based on
 * the documentation of the notebook format at https://nbformat.readthedocs.io.
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
    // Convert each cell
    for (let cell of data.cells) {
      if (cell.cell_type === 'markdown') {
        let html = markdown.md2html(cell.source)
        dom.root().append(html)
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
    doc.content.root().children().each(function () {
      let html = cheerio.html(cheerio(this))
      let md = markdown.html2md(html)
      cells.push({
        cell_type: 'markdown',
        source: md
      })
    })
    return JSON.stringify({
      cells: cells
    })
  }
}

module.exports = DocumentJupyterNotebookConverter
