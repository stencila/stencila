import { Component, EditorSession } from 'substance'
import DocumentEditor from './DocumentEditor'
import DocumentConfigurator from './DocumentConfigurator'
import { importHTML, exportHTML } from './documentConversion'
import JsContext from '../js-context/JsContext'

/*
  Usage:

  ```js
  DocumentPage.mount({
    backend: myBackend,
    archiveURL: 'https://github.com/stencila/stencila.md'
  })
  ```
*/
export default class DocumentPage extends Component {

  didMount() {
    this._loadArchive()
  }

  didUpdate(oldProps, oldState) {
    // archiveUrl has changed
    if (oldProps.archiveURL !== this.props.archiveURL) {
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
    this._saveToArchive()
  }

  _loadArchive() {
    if (this.props.archiveURL) {
      let configurator = new DocumentConfigurator()
      let backend = this.getBackend()
      let archive = backend.getArchive(this.props.archiveURL)
      if (!archive) throw new Error('Could not find archive.')
      archive.readFile('index.html').then((docHTML) => {
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
        // editorSession._url = this.props.archiveURL
        this.setState({
          editorSession: editorSession
        })
      })
    }
  }

  _saveToArchive() {
    if (this.props.archiveURL) {
      let backend = this.getBackend()
      let archive = backend.getArchive(this.props.archiveURL)
      if (!archive) throw new Error('Could not find archive.')
      const editorSession = this.state.editorSession
      if (!editorSession) return
      const doc = editorSession.getDocument()
      const html = exportHTML(doc)
      // TODO at some point we would need to write everything, not just HTML
      archive.writeFile('index.html', 'text/html', html).then(() => {
        console.info('Archive saved.')
      })
    }
  }

  _registerEvents(editorSession) {
    editorSession.on('render', this._onSelectionChange, this, {
      resource: 'selection'
    })
  }

  _unregisterEvents(editorSession) {
    editorSession.off(this)
  }

  _onSelectionChange() {
    let toolGroups = this.refs.editor.toolGroups
    let commandStates = this.state.editorSession.getCommandStates()
    let appState = this.getAppState()
    if (appState) {
      appState.extend({
        commandStates: commandStates,
        toolGroups: toolGroups,
        hasPendingChanges: appState.hasPendingChanges
      })
    }
  }

}
