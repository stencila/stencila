import { Component, EditorSession } from 'substance'
import DocumentEditor from './DocumentEditor'
import DocumentConfigurator from './DocumentConfigurator'
import { importHTML, exportHTML } from './documentConversion'
import JsContext from '../js-context/JsContext'
import debounce from 'lodash.debounce'

/*
  Usage:

  ```js
  DocumentPage.mount({
    backend: myBackend,
    documentId: 'welcome-to-stencila'
  })
  ```
*/
export default class DocumentPage extends Component {

  constructor(...args) {
    super(...args)
    this._saveToBufferDebounced = debounce(this._saveToBuffer, 500)
    this._saveToBufferDebounced.bind(this)
  }

  didMount() {
    this._loadArchive()
  }

  didUpdate(oldProps, oldState) {
    // documentId has changed
    if (oldProps.documentId !== this.props.documentId) {
      this._loadArchive()
    }
    // editor session has changed
    if (oldState.editorSession !== this.state.editorSession) {
      if (oldState.editorSession) {
        this._unregisterEvents(oldState.editorSession)
      }
      if (this.state.editorSession) {
        this._registerEvents(this.state.editorSession)
      }
      this.emit('loaded')
    }
  }

  dispose() {
    if (this.state.editorSession) {
      this._unregisterEvents(this.state.editorSession)
    }
  }

  render($$) {
    let el = $$('div').addClass('sc-document-page')
    let editorSession = this.state.editorSession
    if (editorSession) {
      el.append(
        $$(DocumentEditor, {
          editorSession: editorSession,
          edit: true
        }).ref('editor')
      )
    }
    return el
  }

  getBackend() {
    return this.props.backend
  }

  getAppState() {
    return this.props.appState
  }

  executeCommand(commandName, params) {
    this.state.editorSession.executeCommand(commandName, params)
  }

  save() {
    this._storeBuffer()
  }

  _loadArchive() {
    if (this.props.documentId) {
      let configurator = new DocumentConfigurator()
      let backend = this.getBackend()

      backend.getArchive(this.props.documentId).then((buffer) => {
        buffer.readFile('index.html', 'text/html').then((docHTML) => {
          let doc = importHTML(docHTML)
          let editorSession = new EditorSession(doc, {
            configurator: configurator,
            context: {
              stencilaContexts: {
                'js': new JsContext()
              }
            }
          })
          // enable this to make debugging easier
          // editorSession._url = this.props.documentId
          this.setState({
            buffer,
            editorSession
          })
        })
      })
    }
  }

  /*
    Saves the current buffer to the store backing the document.
  */
  _storeBuffer() {
    if (this.props.documentId) {
      let backend = this.getBackend()
      let appState = this.getAppState()
      const buffer = this.state.buffer

      backend.storeArchive(buffer).then(() => {
        if (appState) {
          appState.extend({
            hasPendingChanges: false,
          })
        }
      }).catch((err) => {
        console.error(err)
        if (appState) {
          appState.extend({
            error: err.message
          })
        }
      })
    }
  }

  _registerEvents(editorSession) {
    editorSession.on('render', this._onSelectionChange, this, {
      resource: 'selection'
    })
    editorSession.on('render', this._onDocumentChange, this, {
      resource: 'document'
    })
  }

  _unregisterEvents(editorSession) {
    editorSession.off(this)
  }

  /*
    HACK: We inspect the document and try to use the first node as the title
  */
  getTitle() {
    let editorSession = this.refs.editor.getEditorSession()
    let doc = editorSession.getDocument()
    let docNodes = doc.get('content').nodes
    let firstNode = doc.get(docNodes[0])
    if (firstNode && firstNode.content) {
      return firstNode.content
    } else {
      return 'Untitled'
    }
  }

  _onSelectionChange() {
    let toolGroups = this.refs.editor.toolGroups
    let commandStates = this.state.editorSession.getCommandStates()
    let appState = this.getAppState()
    let title = this.getTitle()
    if (appState) {
      appState.extend({
        title: title,
        commandStates: commandStates,
        toolGroups: toolGroups
      })
    }
  }

  _onDocumentChange() {
    let appState = this.getAppState()
    if (appState && !appState.get('hasPendingChanges')) {
      appState.extend({
        hasPendingChanges: true
      })
    }
    this._saveToBufferDebounced()
  }

  /*
    Save current in-memory state of the document to buffer
  */
  _saveToBuffer() {
    const editorSession = this.state.editorSession
    const buffer = this.state.buffer
    const doc = editorSession.getDocument()
    const html = exportHTML(doc)
    const documentId = this.props.documentId
    const backend = this.getBackend()

    buffer.writeFile('index.html', 'text/html', html).then(() => {
      return backend.updateManifest(documentId, {
        updatedAt: new Date(),
        title: this.getTitle()
      })
    }).catch((err) => {
      console.error(err)
    })
  }
}
