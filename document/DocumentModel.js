import Document from 'substance/model/Document'

import DocumentConfigurator from './DocumentConfigurator'

/**
 * A Stencila Document data model
 *
 * @class      Document (name)
 * @param      {<type>}  schema  The schema
 */
class DocumentModel extends Document {

  constructor (schema) {
    if (!schema) {
      let configurator = new DocumentConfigurator()
      schema = configurator.getSchema()
    }
    super(schema)

    // Create a root body container node for the document
    this.create({
      type: 'container',
      id: 'content',
      nodes: []
    })
  }

}

export default DocumentModel
