import {HTMLImporter} from 'substance'

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
