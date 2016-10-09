import Document from 'substance/model/Document'

import DocumentConfigurator from './DocumentConfigurator'
var configurator = new DocumentConfigurator()

/**
 * A Stencila Document data model
 *
 * @class      Document (name)
 * @param      {<type>}  schema  The schema
 */
class DocumentModel extends Document {
  constructor (schema) {
    super(schema || DocumentModel.schema)

    // Create a root body container node for the document
    this.create({
      type: 'container',
      id: 'content',
      nodes: []
    })
  }

  static get schema () {
    configurator.getSchema()
  }

  execute (expression, context) {
    context = context || this.contexts[0]
    return context.execute(expression)
  }

  write (expression) {
    return this.contexts[0].write(expression)
  }
}

export default DocumentModel
