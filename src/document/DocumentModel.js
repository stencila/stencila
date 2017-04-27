import {Document} from 'substance'

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

    // Create a container node for the document's content
    this.create({
      type: 'container',
      id: 'content',
      nodes: []
    })

    this.variables = {}
  }

}

export default DocumentModel
