import HTMLImporter from 'substance/model/HTMLImporter'

class TestHTMLImporter extends HTMLImporter {

  constructor (DocumentClass, schema, converters) {
    super({
      DocumentClass: DocumentClass,
      schema: schema,
      converters: converters
    })
  }

  convertDocument (els) {
    this.convertContainer(els, 'content')
  }
}

export default TestHTMLImporter
