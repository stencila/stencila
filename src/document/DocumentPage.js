import { Component, EditorSession } from 'substance'
import DocumentEditor from './DocumentEditor'
import DocumentConfigurator from './DocumentConfigurator'
import { importHTML } from './documentConversion'
import JsContext from '../js-context/JsContext'

let configurator = new DocumentConfigurator()

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

  getBackend() {
    return this.props.backend
  }

  getAppState() {
    return this.props.appState
  }

  executeCommand(commandName, params) {
    this.state.editorSession.executeCommand(commandName, params)
  }

  didMount() {
    let backend = this.getBackend()
    let archive = backend.getArchive(this.props.archiveURL)
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

      this.setState({
        editorSession: editorSession
      })
    })
  }

  didUpdate(oldProps, oldState) {
    if (oldState.editorSession) {
      this._unregisterEvents(oldState.editorSession)
    }
    if (this.state.editorSession) {
      this._registerEvents(this.state.editorSession)
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

}
