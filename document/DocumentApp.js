import Component from 'substance/ui/Component'
import DocumentSession from 'substance/model/DocumentSession'
import CollabClient from 'substance/collab/CollabClient'
import CollabSession from 'substance/collab/CollabSession'
import WebSocketConnection from 'substance/collab/WebSocketConnection'

import RemoteDocument from 'stencila/src/document/RemoteDocument'

import DocumentConfigurator from './DocumentConfigurator'
var configurator = new DocumentConfigurator()
import {importJSON, importHTML} from './documentConversion'

import VisualEditor from './editors/visual/VisualEditor'
import CodeEditor from './editors/code/CodeEditor'

import code from '../utilities/code/index'

/**
 * User application for a Stencila Document
 *
 * @class      DocumentApp (name)
 */
class DocumentApp extends Component {

  constructor (...args) {
    super(...args)

    // Bind to events
    this.handleActions({
      'view-toggle': this.toggleView,
      'reveal-toggle': this.toggleReveal,
      'comment-toggle': this.toggleComment,
      'edit-toggle': this.toggleEdit
    })
  }

  getInitialState () {
    // Initially, if in edit mode, then also turn on reveal mode
    // See also `this.toggleEdit`
    var view = this.props.view || 'visual'
    var edit = (this.props.edit === '1') || this.props.local
    var reveal = (this.props.reveal === '1') || edit
    return {
      view: view,
      reveal: reveal,
      edit: edit,
      documentSession: null,
      message: null
    }
  }

  /**
  * Render the application
  *
  * @param      {Function}  $$ Function for creating virtual nodes
  * @return     {VirtualNode}  Virtual node to be added to the DOM
  */
  render ($$) {
    var el = $$('div').addClass('sc-document-app')

    if (this.state.documentSession) {
      var session = null
      var copy = null
      if (this.props.copy) {
        copy = {
          name: this.props.copy,
          people: Object.keys(this.state.documentSession.collaborators).length + 1
        }
      }
      var editorProps = {
        // Document state
        session: session,
        copy: copy,
        view: this.state.view,
        reveal: this.state.reveal,
        comment: this.state.comment,
        edit: this.state.edit,
        // Other required props
        documentSession: this.state.documentSession,
        configurator: configurator
      }

      var view
      if (this.state.view === 'visual') {
        view = $$(VisualEditor, editorProps).ref('visualEditor')
      } else {
        view = $$(CodeEditor, editorProps).ref('codeEditor')
      }
      el.append(view)
    } else {
      el
        .addClass('sm-loading')
        .append(
          $$('i')
            .addClass('fa fa-spinner fa-pulse fa-fw')
        )
    }

    if (this.state.message) {
      el
        .addClass('sm-message')
        .append(
          $$('p')
            .addClass('se-message')
            .text(this.state.message.string)
        )
    }

    return el
  }

  didMount () {
    var doc
    var documentSession

    if (this.props.format === 'html') {
      // Import the HTML provided
      doc = importHTML(this.props.data)

      // Create a new local document session
      documentSession = new DocumentSession(doc)
      // For code compatability with a `CollabSession` ...
      documentSession.config = {
        user: null,
        rights: null
      }
    } else {
      var user = this.props.data.user
      var rights = this.props.data.rights
      var collabUrl = this.props.data.collabUrl
      var snapshot = this.props.data.snapshot

      // Import the JSON provided
      doc = importJSON(snapshot.data)

      // Create a new collaborative document session
      var collabConn = new WebSocketConnection({
        wsUrl: collabUrl
      })
      var collabClient = new CollabClient({
        connection: collabConn
      })
      documentSession = new CollabSession(doc, {
        documentId: snapshot.documentId,
        version: snapshot.version,
        collabClient: collabClient,
        user: user,
        rights: rights
      })
    }

    documentSession.remote = new RemoteDocument(this.props.url)

    code.loadAce()

    // Define execution contexts for document
    doc.contexts = [
      // new JavascriptContext()
    ]

    // Extend state to trigger rerendering
    this.extendState({
      doc: doc,
      documentSession: documentSession
    })
  }

  /**
   * Toggle the view
   */
  toggleView (editor) {
    this.extendState({
      view: (this.state.view === 'visual') ? 'code' : 'visual'
    })
  }

  /**
   * Toggle the `reveal` state
   */
  toggleReveal () {
    this.extendState({
      reveal: !this.state.reveal
    })
  }

  /**
   * Toggle the `comment` state
   */
  toggleComment () {
    var comment = !this.state.comment
    if (comment) {
      this.switchCopy('live')
    }
    this.extendState({
      comment: comment
    })
  }

  /**
   * Toggle the `edit` state. If edit mode is getting turned on
   * then reveal mode is also automatically turned on.
   */
  toggleEdit () {
    var edit = !this.state.edit
    if (edit) {
      this.switchCopy('live')
    }
    this.extendState({
      reveal: edit || this.state.reveal,
      comment: edit || this.state.comment,
      edit: edit
    })
  }

  /**
   * Switch to a different copy (if necessary)
   *
   * @param      {string}  copy   The copy
   */
  switchCopy (copy) {
    if (this.props.copy !== copy) {
      this.extendState({
        documentSession: null
      })
      window.location = window.location + '@' + copy
    }
  }
}

export default DocumentApp
