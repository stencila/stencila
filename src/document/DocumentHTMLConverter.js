import { DefaultDOMElement } from 'substance'
import beautify from 'js-beautify'

export default class DocumentHTMLConverter {

  /*
    Read a storer (source file layout) and store to a buffer (internal Stencila
    file format)

    Original fileName is needed because otherwise we don't know what to read
    from the storer.

    TODO: Binaries could be included, which we should also consider.
  */

  importDocument(storer, buffer) {
    let mainFilePath = storer.getMainFilePath()
    let manifest = {
      "type": "document",
      "storage": {
        "external": storer.isExternal(),
        "storerType": storer.getType(),
        "archivePath": storer.getArchivePath(),
        "mainFilePath": mainFilePath,
        "contentType": "html",
      },
      "createdAt": new Date(),
      "updatedAt": new Date()
    }
    return storer.readFile(
      mainFilePath,
      'text/html'
    ).then((html) => {
      manifest.title = this._extractTitle(html)
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

  /*
    Takes a buffer and writes back to the storer
  */
  exportDocument(buffer, storer, options = {}) {
    if (options.beautify !== false) options.beautify = true

    let mainFilePath = storer.getMainFilePath()
    return buffer.readFile('index.html', 'text/html').then((html) => {
      // Remove the `data-id` attribute that is added by the Substance exporter
      // to each node element
      html = html.replace(/ data-id=".+?"/g, '')
      if (options.beautify) {
        // Beautification. Because right now, some editing of HTML source is necessary
        // because the visual editor can't do everything e.g tables
        // See options at https://github.com/beautify-web/js-beautify/blob/master/js/lib/beautify-html.js
        html = beautify.html(html, {
          'indent_inner_html': false,
          'indent_size': 2,
          'indent_char': ' ',
          'wrap_line_length': 0, // disable wrapping
          'brace_style': 'expand',
          'preserve_newlines': true,
          'max_preserve_newlines': 5,
          'indent_handlebars': false,
          'extra_liners': ['/html']
        })
      }
      return storer.writeFile(mainFilePath, 'text/html', html)
    })
  }

  _extractTitle(html) {
    var htmlDoc = DefaultDOMElement.parseHTML(html)
    let titleEl = htmlDoc.find('div[data-title]')
    return titleEl ? titleEl.textContent : 'Untitled'
  }
}

DocumentHTMLConverter.match = function(fileName) {
  return fileName.indexOf('.html') >= 0
}
