import { comment as languageComment } from '../language'
import DocumentMarkdownConverter from './DocumentMarkdownConverter'

/**
A Document converter for XMarkdown.

XMarkdown is our name for RMarkdown-like formats, that is, RMarkdown but extended to language
X, where X includes Python, Javascript, etc.

This is a preliminary implementation and not all of the following conversions are enabled yet.

In RMarkdown, R code is embedded in "code chunks". There are two types of code chunks: inline and block.
In XMarkdown, we allow both inline and block chunks to be defined in various languages using
our usual language labels e.g. ``r``, ``py``, ``js``.

Inline code chunks, equivalent to Stencila's output cells, are declared using Markdown code spans
prefixed by the language label e.g.

    The answer is `r x`

Block code chunks, equivalent to Stencila's code cells, are declared using Markdown fenced code blocks
with attributes prefixed by the language label and, optionally, a chunk label and other options e.g.

    ```{r myplot, fig.width=6, fig.height=7}
    plot(x,y)
    ```

Here ``myplot`` is the chunk label and ```fig.width=6, fig.height=7``` are chunk options.
A list of chunk options, recognised by the RMarkdown rendering enging, Knitr,
is available at http://yihui.name/knitr/options/.

For RMarkdown documents, to maintain compatability with Knitr, options are converted to
Stencila execute directive settings as follows

- eval=FALSE : do FALSE
- echo=TRUE : show TRUE
- fig.height=6 : height 6
- fig.width=7 : width 7

**/
export default class DocumentXMarkdownConverter extends DocumentMarkdownConverter {

  /**
   * Is a file name to be converted using this converter? 
   * 
   * @param  {string} fileName - Name of file
   * @return {boolean} - Is the file name matched?
   */
  static match (fileName, extensions) {
    let dot = fileName.lastIndexOf('.')
    let extension = fileName.substring(dot+1).toLowerCase()
    return extensions.indexOf(extension) >= 0
  }

  /**
   * Convert XMarkdown to Stencila Document HTML
   *
   * @param {string} xmd - XMarkdown content
   * @return {string} - HTML content
   */
  importContent (xmd) {
    // Covert XMarkdown to Markdown
    let md = ''
    for(let line of xmd.split('\n')) {
      // XMarkdown code chunks to Stencila Markdown code cells
      let match = line.match(/^```\s*{([a-z]+)\s*(.*)}/)
      if (match) {
        let language = match[1]
        let comment = languageComment(language)
        md += '```' + language + '\n'
        md += `${comment}! global\n`
        let options = match[2]
        if (options) md += `${comment}: ${options}\n`
      } else {
        md += line + '\n'
      }
    }
    // Convert Markdown to HTML
    return super.importContent(md)
  }
  /**
   * Convert a Stencila Document HTML to XMarkdown
   *
   * @param {string} html - HTML content
   * @return {string} - XMarkdown content
   */
  exportContent (html) {
    // Covert HTML to Markdown
    let md = super.exportContent(html)
    // Convert Markdown to XMarkdown
    let xmd = ''
    let lines = md.split('\n')
    for(let index = 0; index < lines.length; index++) {
      let line = lines[index]
      // Stencila Markdown code cells to XMarkdown code chunks 
      let match = line.match(/^```([a-z]+)/)
      if (match) {
        let language = match[1]
        let comment = languageComment(language)
        let first = lines[index+1]
        if (first) {
          // Check for shebang on first line
          let shebang = first.match(`^${comment}!\\s*global`)
          if (shebang) {
            let spec = '```{' + language
            index += 1
            // Check for options on second line
            let second = lines[index+1]
            let options = second.match(`^${comment}:\\s*(.*)`)
            if (options) {
              spec += ' ' + options[1]
              index += 1
            }
            xmd += spec + '}\n'
          }
        }
      } else {
        xmd += line + '\n'
      }
    }
    return xmd
  }

}
