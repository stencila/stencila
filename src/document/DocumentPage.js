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

  render($$) {
    let el = $$('div').addClass('sc-document-page')
    let editorSession = this.state.editorSession
    if (editorSession) {
      el.append(
        $$(DocumentEditor, {
          editorSession: editorSession,
          edit: true
        })
      )
    }
    return el
  }

}
