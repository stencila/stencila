import HTMLImporter from 'substance/model/HTMLImporter'

class TestHTMLImporter extends HTMLImporter {

  constructor (DocumentClass, converters) {
    super({
      DocumentClass: DocumentClass,
      schema: DocumentClass.schema,
      converters: converters
    })
  }

  convertDocument (els) {
    this.convertContainer(els, 'content')
  }
}

export default TestHTMLImporter
