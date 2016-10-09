import HTMLExporter from 'substance/model/HTMLExporter'

class TestHTMLExporter extends HTMLExporter {

  constructor (DocumentClass, converters) {
    super({
      DocumentClass: DocumentClass,
      schema: DocumentClass.schema,
      converters: converters
    })
  }

  exportDocument (doc) {
    var bodyNodes = this.convertContainer(doc.get('content'))
    var wrapper = this.$$('div').append(bodyNodes)
    return wrapper.html()
  }
}

export default TestHTMLExporter
