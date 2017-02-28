import { Component, EditorSession } from 'substance'

import DocumentConfigurator from './DocumentConfigurator'
var configurator = new DocumentConfigurator()
import {importJSON, importHTML} from './documentConversion'

import VisualEditor from './editors/visual/VisualEditor'
import CodeEditor from './editors/code/CodeEditor'

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
    let view = this.props.view || 'visual'
    let edit = this.props.edit
    if (typeof edit === 'undefined' && this.props.local) edit = '1'
    edit = edit === '1'
    let reveal = (this.props.reveal === '1') || edit
    let naked = (this.props.naked === '1') || reveal
    return {
      view: view,
      naked: naked,
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
      var version = null
      if (this.props.version) {
        version = {
          name: this.props.version,
          // TODO Fix this. Will only have collaborators on documentSession if this is a "@live" version
          people: 0 // Object.keys(this.state.documentSession.collaborators).length + 1
        }
      }
      var editorProps = {
        // Document state
        version: version,
        naked: this.state.naked,
        view: this.state.view,
        reveal: this.state.reveal,
        comment: this.state.comment,
        edit: this.state.edit,
        // Other required props
        editorSession: this.state.documentSession,
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

    // FIXME: we more code here establishing a CollabSession.
    // We removed it as this should be provided by the integrating
    // environment
    if (this.props.format === 'html') {
      // Import the HTML provided
      doc = importHTML(this.props.data)
    } else if (this.props.format === 'json') {
      doc = importJSON(this.props.data)
    }
    // Create a new local document session
    documentSession = new EditorSession(doc, {
      configurator: configurator
    })
    // For code compatability with a `CollabSession` ...
    documentSession.config = {
      user: null,
      rights: null
    }
    // TODO: why is this necessary?
    doc.documentSession = documentSession
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
      // FIXME This is turned off for cases like file:// where there will be no live version
      // available
      // this.switchCopy('live')
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
