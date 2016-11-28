import Document from 'substance/model/Document'

import DocumentConfigurator from './DocumentConfigurator'
import SessionClient from '../session/SessionClient'

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

    // Create a container node for the document's sessions
    this.create({
      type: 'container',
      id: 'sessions',
      nodes: []
    })

    // The data pipeline
    this.pipeline = {
      'default': null // The default pipe
    }
  }

  /**
   * Get the document's session for a particular language
   *
   * If no existing session for the language, then one will be created.
   *
   * @param  {[type]} language [description]
   * @return {[type]}          [description]
   */
  session (language) {
    // TODO put this into a new container node for "sessions" - could be reused in Sheets
    // TODO check for existing session for language
    let sessionNodes = this.get('sessions').getChildren()
    if (sessionNodes.length) {
      let sessionNode = sessionNodes[0] // TODO - search for right type of session
      let sessionClient = new SessionClient(sessionNode.url)
      return Promise.resolve(sessionClient)
    } else {
      return new Promise((resolve, reject) => {
        this.host.new('session-' + language).then(sessionClient => {
          this.documentSession.transaction(tx => {
            let sessionNode = tx.create({
              type: 'session',
              // TODO - populate with the session data e.g. language, url etc
              url: sessionClient._url
            })
            let sessions = tx.get('sessions')
            sessions.show(sessionNode)
            resolve(sessionClient)
          })
        })
      })
    }
  }

}

export default DocumentModel
