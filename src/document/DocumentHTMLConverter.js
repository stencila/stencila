import { DefaultDOMElement } from 'substance'
import beautify from 'js-beautify'

export default class DocumentHTMLConverter {

  /**
   * Is a file name to be converted using this converter? 
   * 
   * @param  {string} path - Name of file
   * @return {boolean} - Is the file name matched?
   */
  static match (path, storer) {
    return path.slice(-5) === '.html'
  }

  /*
    Read a storer (source file layout) and store to a buffer (internal Stencila
    file format)

    Original fileName is needed because otherwise we don't know what to read
    from the storer.

    TODO: Binaries could be included, which we should also consider.
  */

  import(path, storer, buffer) {
    return storer.readFile(path).then((html) => {
      html = html || `
        <!DOCTYPE html>
        <html>
          <head>
            <title></title>
          </head>
          <body>
            <main>
              <div id="data" data-format="html">
                <div class="content"><p>Start here!</p></div>
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

  /*
    Takes a buffer and writes back to the storer
  */
  export(path, storer, buffer, options = {}) {
    if (options.beautify !== false) options.beautify = true

    return buffer.readFile('index.html').then((html) => {
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
          'extra_liners': [],
          'end_with_newline': true
        })
      }
      return storer.writeFile(path, html)
    })
  }
}
