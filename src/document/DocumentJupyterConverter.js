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
 */
export default class DocumentJupyterConverter {

  /**
   * Is a file name to be converted using this converter? 
   * 
   * @param  {string} fileName - Name of file
   * @return {boolean} - Is the file name matched?
   */
  static match (fileName) {
    return fileName.slice(-6) === '.ipynb'
  }

  // Import a document from storer to buffer
  // This method is likely to be refactored into a base class method
  // since it is very similar for all single-file based converters e.g. DocumentMarkdownConverter
  importDocument(storer, buffer) {
    let mainFilePath = storer.getMainFilePath()
    let manifest = {
      "type": "document",
      "storage": {
        "external": storer.isExternal(),
        "storerType": storer.getType(),
        "archivePath": storer.getArchivePath(),
        "mainFilePath": mainFilePath,
        "contentType": "ipynb",
      },
      "createdAt": new Date(),
      "updatedAt": new Date()
    }
    return storer.readFile(
      mainFilePath,
      'text/html'
    ).then(json => {
      let html = `<!DOCTYPE html>
<html>
  <head>
    <title></title>
  </head>
  <body>
    <main>
      <div id="data" data-format="html">
        <div class="content">${this.importContent(json)}</div>
      </div>
    </main>
  </body>
</html>`
      return buffer.writeFile(
        'index.html',
        'text/html',
        html
      )
    }).then(() => {
      return buffer.writeFile(
        'stencila-manifest.json',
        'application/json',
        JSON.stringify(manifest, null, '  ')
      )
    }).then(() => {
      return manifest
    })
  }

  // Export a document from buffer to storer
  // This method is likely to be refactored into a base class method
  // since it is very similar for all single-file based converters e.g. DocumentMarkdownConverter
  exportDocument(buffer, storer) {
    return buffer.readFile('index.html', 'text/html').then((html) => {
      let content = DefaultDOMElement.parseHTML(html).find('.content')
      if (!content) throw new Error('No div.content element in HTML!')
      let ipynb = this.exportContent(content.getInnerHTML())
      return storer.writeFile(storer.getMainFilePath(), 'text/json', ipynb)
    })
  }


  /**
   * Convert from a Jupyter Notebook to a Stencila Document
   *
   * @memberof document/jupyter
   *
   * @param  {string|object} json - Content of Jupyter Notebook (JSON string of Object)
   * @return {string} Content of Stencila Document (HTML)
   */
  importContent (json) {
    // Get notebook data, by parsing JSON string if necessary
    let data = (typeof json === 'string') ? JSON.parse(json) : json

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
    return html
  }

  /**
   * Convert to a Jupyter Notebook from a Stencila Document
   *
   * @memberof document/jupyter
   *
   * @param {string} html - Document HTML string
   * @return {string|object} - Jupyter notebook JSON or object
   */
  exportContent (html, options) {
    options = options || {}
    if (options.stringify !== false) options.stringify = true
    if (options.pretty !== false) options.pretty = true

    // Initial, empty notebook object
    let nb = {
      cells: [],
      metadata: {}, // TODO lang from executes - ensure only one
      nbformat: 4,
      nbformat_minor: 2
    }

    // Create a DOM document to iterate over
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

    return content
  }

}
