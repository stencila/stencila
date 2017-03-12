/**
 * Jupyter notebook coonverter for the `Document` class
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

// Use Substance's DOM implementation. See
//    https://github.com/substance/substance/blob/develop/dom/DOMElement.js
// for the API
import {DefaultDOMElement} from 'substance'

//import * as context from '../context/context'
//import * as markdown from '../utilities/markdown'

/**
 * Convert from a Jupyter Notebook to a Stencila Document
 *
 * @memberof document/jupyter
 *
 * @param  {string} content - Content of Jupyter Notebook (JSON)
 * @return {string} Content of Stencila Document (HTML)
 */
export function import_ (content) {
  // Get notebook data, by parsing JSON string if necessary
  let data = (typeof content === 'string') ? JSON.parse(content) : content

  let doc = DefaultDOMElement.createDocument('html')
  let $$ = doc.createElement.bind(doc)
  let root = $$('div')

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
  let langCode = 'foo' // context.language.code(lang) || ''

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
      let html = source //markdown.md2html(source)
      root.append(html)
    } else if (cell.cell_type === 'code') {
      let $execute = $$('div')
      $execute.attr('data-execute', langCode)

      $execute.append(
        $$('pre').attr('data-code', '').text(source)
      )

      let outputs = cell.outputs
      if (outputs) {
        for (let output of outputs) {
          let $result
          let type = output.output_type
          if (type === 'execute_result' || type === 'display_data') {
            let mimebundle = output.data
            // Currently just use the first mime type (there may be multiple
            // formats for the output)
            let mimetype = Object.keys(mimebundle)[0]
            let value = mimebundle[mimetype]
            if (value.constructor.name === 'Array') value = value.join('')

            // TODO : security, sanitize?
            let type
            let format
            if (mimetype === 'image/png') {
              type = 'img'
              let match = mimetype.match('^image/([a-z]+)$')
              format = match ? match[1] : ''
              $result = $$('img').attr('src', `data:${mimetype};base64,${value}`)
            } else if (mimetype === 'image/svg+xml') {
              type = 'img'
              format = 'svg'
              $result = $$('div').html(value)
            } else if (mimetype === 'text/html') {
              type = 'dom'
              format = 'html'
              $result = $$('div').html(value)
            } else if (mimetype === 'text/latex') {
              type = 'math'
              format = 'latex'
              if (value.substring(0, 2) === '$$') value = value.substring(2)
              if (value.slice(-2) === '$$') value = value.slice(0, -2)
              $result = $$('pre').text(value)
            } else {
              type = 'str'
              format = 'text'
              $result = $$('pre').text(value)
            }
            $result.attr('data-result', type)
            $result.attr('data-format', format)
          } else if (type === 'stream') {
            $result = $$('pre')
            if (output.name === 'stderr') $result.attr('data-errors', true)
            else $result.attr('data-result', 'str')
            $result.text(output.text.join(''))
          } else if (type === 'error') {
            $result = $$('pre').attr('data-errors', true)
            $result.text(output.ename + ': ' + output.evalue + '\n\n' + output.traceback)
          }

          if ($result) $execute.append($result)
        }
      }

      root.append($execute)
    }
  }
  return root.html()
}

/**
 * Convert to a Jupyter Notebook from a Stencila Document
 *
 * @memberof document/jupyter
 *
 * @param  {Document} doc - Document to dump
 * @param  {Object} options - Any options (see implementations for those available)
 * @returns {String} - Content of the document as HTML
 */
export function export_ (doc, options) {
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
    let $this = $(this)
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
      html += $.html($this)
    }
  })
  flush()

  if (options.stringify) {
    return JSON.stringify(nb, null, options.pretty ? '  ' : null)
  } else {
    return nb
  }
}
