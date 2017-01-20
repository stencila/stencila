var DocumentMarkdownConverter = require('./DocumentMarkdownConverter')

class Document {
  constructor (options) {
    this.content = ''
  }

  load (format, content, options) {
    this.converter(format).load(this, content, options)
  }

  dump (format, options) {
    return this.converter(format).dump(this, options)
  }

  converter (format) {
    if (format === 'md') {
      return new DocumentMarkdownConverter()
    }
  }
}

module.exports = Document
