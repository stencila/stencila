import {HTMLExporter} from 'substance'

class TestHTMLExporter extends HTMLExporter {

  constructor (DocumentClass, schema, converters) {
    super({
      DocumentClass: DocumentClass,
      schema: schema,
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
