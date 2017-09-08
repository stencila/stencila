import { Component, EditorSession } from 'substance'
import DocumentEditor from './DocumentEditor'
import DocumentConfigurator from './DocumentConfigurator'
import CellEngine from './CellEngine'
import { importHTML, exportHTML } from './documentConversion'
import debounce from 'lodash.debounce'

import DocumentHTMLConverter from './DocumentHTMLConverter'
import DocumentMarkdownConverter from './DocumentMarkdownConverter'
import DocumentRMarkdownConverter from './DocumentRMarkdownConverter'
import DocumentJupyterConverter from './DocumentJupyterConverter'

const CONVERTERS = [
  DocumentHTMLConverter,
  DocumentMarkdownConverter,
  DocumentRMarkdownConverter,
  DocumentJupyterConverter
]

/*
  Usage:

  ```js
  DocumentPage.mount({
    address: 'memory://welcome-to-stencila',
    host: new Host(...)
  })
  ```
*/
export default class DocumentPage extends Component {

  constructor(...args) {
    super(...args)
    this._bufferDebounced = debounce(this._buffer, 500)
    this._bufferDebounced.bind(this)
  }

  didMount() {
    this._open()
  }

  didUpdate(oldProps, oldState) {
    // documentId has changed
    if (oldProps.documentId !== this.props.documentId) {
      this._open()
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

  getAppState() {
    return this.props.appState
  }

  executeCommand(commandName, params) {
    this.state.editorSession.executeCommand(commandName, params)
  }

  save() {
    return this._save()
  }

  discard() {
    return this._discard()
  }

  /**
   * Open the document from a Storer
   */
  _open() {
    let address = this.props.address
    if (address) {
      let host = this.props.host
      host.createStorer(address).then((storer) => {
        host.createBuffer(storer).then((buffer) => {
          storer.getInfo().then((info) => {
            // Get the directory, main file and file list from the storer
            let {dir, main, files} = info

            // If no main file is specified in the storer...
            if (!main) {
              // ...find a main file
              for (let name of ['main', 'index', 'README']) {
                let regex = new RegExp('^' + name + '\\.')
                let matched = files.filter(item => item.match(regex))
                if (matched.length) {
                  main = matched[0]
                  break
                }
              }
              // ...fallback to the first file
              if (!main) main = files[0]
            }
            if (!main) return Promise.reject(new Error(`No main file found for address "${address}"`))

            // Find the converter for the main file
            let Converter
            for (let converter of CONVERTERS) {
              if (converter.match(main, storer)) {
                Converter = converter
                break
              }
            }
            if (!Converter) return Promise.reject(new Error(`No converter for main file "${main}"`))

            // Convert the main file...
            let converter = new Converter()
            converter.import(main, storer, buffer).then(html => {

              // ...then, instantiate a document, editor and cell engine
              let doc = importHTML(html)
              let editorSession = new EditorSession(doc, {
                configurator: new DocumentConfigurator()
              })
              let cellEngine = new CellEngine(editorSession, host, dir)

              this.setState({
                host,
                main,
                storer,
                buffer,
                converter,
                editorSession,
                cellEngine
              })
            })
          })
        })
      })
    }
  }

  /*
    Save current in-memory state of the document to buffer
  */
  _buffer() {
    const editorSession = this.state.editorSession
    const doc = editorSession.getDocument()
    const html = exportHTML(doc)
    //const documentId = this.props.documentId

    const buffer = this.state.buffer
    return buffer.writeFile('index.html', html).then(() => {
      //return backend.updateManifest(documentId, {
      //  hasPendingChanges: true
      //})
    })
  }

  /*
    Discard pending changes
  */
  _discard() {
    const buffer = this.state.buffer
    return buffer.clear()
  }

  /*
    Saves the current buffer to the store backing the document.
  */
  _save() {
    const {converter, main, storer, buffer} = this.state
    return converter.export(main, storer, buffer)
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
    this._bufferDebounced()
  }

  _updateAppState(props) {
    let appState = this.getAppState()
    if (appState) {
      appState.extend(props)
    }
  }

}
