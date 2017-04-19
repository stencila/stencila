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
    this._loadBuffer()
  }

  didUpdate(oldProps, oldState) {
    // documentId has changed
    if (oldProps.documentId !== this.props.documentId) {
      this._loadBuffer()
    }
    // editor session has changed
    if (oldState.editorSession !== this.state.editorSession) {
      if (oldState.editorSession) {
        this._unregisterEvents(oldState.editorSession)
      }
      if (this.state.editorSession) {
        this._registerEvents(this.state.editorSession)
      }
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
    return this._storeBuffer()
  }

  discard() {
    return this._discardBuffer()
  }

  _loadBuffer() {
    if (this.props.documentId) {
      let configurator = new DocumentConfigurator()
      let backend = this.getBackend()

      backend.getBuffer(this.props.documentId).then((buffer) => {
        buffer.readFile('index.html', 'text/html').then((docHTML) => {
          let doc = importHTML(docHTML)
          let editorSession = new EditorSession(doc, {
            configurator: configurator,
            context: {
              host: this.props.host
            }
          })

          return buffer.readFile('stencila-manifest.json', 'application/json').then((manifest) => {
            manifest = JSON.parse(manifest)
            this._updateAppState({
              hasPendingChanges: manifest.hasPendingChanges,
              title: manifest.title
            })

            // enable this to make debugging easier
            // editorSession._url = this.props.documentId
            this.setState({
              buffer,
              editorSession
            })
          })
        })
      })
    }
  }

  /*
    Discard pending changes
  */
  _discardBuffer() {
    const buffer = this.state.buffer
    const backend = this.getBackend()
    return backend.discardBuffer(buffer, this.props.documentId)
  }

  /*
    Saves the current buffer to the store backing the document.
  */
  _storeBuffer() {
    let documentId = this.props.documentId
    if (documentId) {
      let backend = this.getBackend()
      let appState = this.getAppState()
      const buffer = this.state.buffer
      return backend.storeBuffer(buffer).then(() => {
        if (appState) {
          this._updateAppState({
            hasPendingChanges: false
          })
        }
        return backend.updateManifest(documentId, {
          title: this.getTitle(),
          hasPendingChanges: false,
          updatedAt: new Date()
        })
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
    let labelProvider = this.refs.editor.labelProvider
    let commandStates = this.state.editorSession.getCommandStates()
    let title = this.getTitle()

    this._updateAppState({
      title: title,
      commandStates: commandStates,
      labelProvider: labelProvider,
      toolGroups: toolGroups
    })
  }

  _onDocumentChange() {
    this._updateAppState({
      hasPendingChanges: true
    })
    this._saveToBufferDebounced()
  }

  _updateAppState(props) {
    let appState = this.getAppState()
    if (appState) {
      appState.extend(props)
    }
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

    return buffer.writeFile('index.html', 'text/html', html).then(() => {
      return backend.updateManifest(documentId, {
        hasPendingChanges: true
      })
    }).catch((err) => {
      console.error(err)
    })
  }
}
