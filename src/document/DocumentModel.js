import Document from 'substance/model/Document'

import DocumentConfigurator from './DocumentConfigurator'
import SessionClient from '../session/SessionClient'
import SessionClientJs from '../session/SessionClientJs'

import Input from './nodes/input/Input'
import Select from './nodes/select/Select'
import Output from './nodes/output/Output'
import Execute from './nodes/execute/Execute'
import Include from './nodes/include/Include'

import {exportHTML} from './documentConversion'

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

    this.variables = {}
  }

  /**
   * Initialize the document
   *
   * Set up the document's variables from any `Input` directives within it
   * and the dependency graph from any `Execute` directives
   */
  initialize () {
    for (let id in this.getNodes()) {
      let node = this.get(id)
      if (node instanceof Input || node instanceof Select) {
        if (node.value) {
          this.variables[node.name] = node.getPack()
        }
      }
    }
    this.refresh()
  }

  /**
   * Refresh the document
   */
  refresh (variable) {
    // TODO : This is a simple hack. Needs a dependency graph and a topological sort
    // for order of refresh and detection of circularities
    for (let id in this.getNodes()) {
      let node = this.get(id)
      if (node instanceof Execute) {
        if (typeof variable === 'undefined' || node.input.split(',').indexOf(variable) > -1) {
          node.refresh()
        }
      }
    }
    for (let id in this.getNodes()) {
      let node = this.get(id)
      if (node instanceof Output) {
        node.refresh()
      }
    }

    // Refresh all include nodes
    for (let id in this.getNodes()) {
      let node = this.get(id)
      if (node instanceof Include) {
        node.refresh()
      }
    }
  }

  /**
   * Set one of the document's variables
   *
   * @param {[type]} name  [description]
   * @param {[type]} value [description]
   */
  setVariable (name, value) {
    this.variables[name] = value
    this.refresh(name)
  }

  /**
   * Get one of the document's variables
   *
   * @param {[type]} name  [description]
   * @param {[type]} value [description]
   */
  getVariable (name) {
    return this.variables[name]
  }

  /**
   * Get the document's session for a particular language
   *
   * If no existing session for the language, then one will be created.
   *
   * @param  {[type]} language [description]
   * @return {[type]}          [description]
   */
  getSession (language) {
    // TODO put this into a new container node for "sessions" - could be reused in Sheets
    // TODO check for existing session for language
    let match = null
    let sessionNodes = this.get('sessions').getChildren()
    sessionNodes.forEach(sessionNode => {
      if (sessionNode.language === language) {
        match = sessionNode
      }
    })
    if (match) {
      return Promise.resolve(new SessionClient(match.url))
    } else {
      if (language === 'js') {
        return Promise.resolve(new SessionClientJs())
      } else {
        return this.host.open(`+${language}-session`).then(sessionClient => {
          this.documentSession.transaction(tx => {
            let sessionNode = tx.create({
              type: 'session',
              // TODO - populate with the session data e.g. language, url etc
              url: sessionClient._url,
              language: language
            })
            let sessions = tx.get('sessions')
            sessions.show(sessionNode)
            return sessionClient
          })
        })
      }
    }
  }

  get html () {
    return exportHTML(this)
  }

  save () {
    return this.delegate.call('save', {
      content: this.html,
      format: 'html'
    })
  }

}

export default DocumentModel
