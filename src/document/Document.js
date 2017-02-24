const cheerio = require('cheerio')

const Component = require('../component/Component')
const DocumentHtmlConverter = require('./DocumentHtmlConverter')
const DocumentMarkdownConverter = require('./DocumentMarkdownConverter')
const DocumentJupyterNotebookConverter = require('./DocumentJupyterNotebookConverter')

/**
 * The Stencila Document class
 */
class Document extends Component {

  /**
   * Construct a document
   */
  constructor (address) {
    super(address)

    this.content = cheerio.load('')
  }

  /**
   * Formats supported by this class
   */
  static get formats () {
    return ['html', 'md']
  }

  static default (name) {
    return {
      format: 'html'
    }[name] || super.default(name)
  }

  /**
   * Get the `Document` converter for a format
   *
   * @param {string} format – The format needing conversion
   * @returns {converter} - A converter object
   */
  static converter (format) {
    format = format || this.default('format')

    if (format === 'html') {
      return new DocumentHtmlConverter()
    } else if (format === 'md' || format === 'rmd') {
      return new DocumentMarkdownConverter()
    } else if (format === 'ipynb') {
      return new DocumentJupyterNotebookConverter()
    } else {
      return super.converter(format)
    }
  }

  get html () {
    return this.dump('html')
  }

  set html (content) {
    return this.load(content, 'html')
  }

  get md () {
    return this.dump('md')
  }

  set md (content) {
    return this.load(content, 'md')
  }

  get ipynb () {
    return this.dump('ipynb')
  }

  set ipynb (content) {
    return this.load(content, 'ipynb')
  }

}

module.exports = Document
