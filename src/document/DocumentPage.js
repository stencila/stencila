import { Component, EditorSession, DefaultDOMElement } from 'substance'
import { JATSImporter, JATSExporter, TextureConfigurator } from 'substance-texture'
import EditorPackage from './EditorPackage'


export default class DocumentPage extends Component {

  constructor(parent, props) {
    super(parent, props)
    this.configurator = new TextureConfigurator()
    this.configurator.import(EditorPackage)
    this.jatsImporter = new JATSImporter()
    this.jatsExporter = new JATSExporter()
  }

  didMount() {
    this._loadDocument()
  }

  didUpdate(oldProps, oldState) {
    // documentId has changed
    if (oldProps.documentPath !== this.props.documentPath) {
      this._loadDocument()
    }
    // editor session has changed
    if (oldState.editorSession !== this.state.editorSession) {
      if (oldState.editorSession) {
        // this._unregisterEvents(oldState.editorSession)
      }
      if (this.state.editorSession) {
        // this._registerEvents(this.state.editorSession)
      }
    }
  }

  dispose() {
    if (this.state.editorSession) {
      // this._unregisterEvents(this.state.editorSession)
    }
  }

  getConfigurator() {
    return this.configurator
  }

  render($$) {
    let el = $$('div').addClass('sc-document-page')

    if (this.state.loadingError) {
      el.append(this.state.loadingError)
    }

    if (this.state.editorSession) {
      el.append(
        $$(EditorPackage.Editor, {
          // documentId: this.props.documentId,
          editorSession: this.state.editorSession
        })
      )
    } else if (this.state.importerErrors) {
      console.error('JATS Import Error', this.state.importerErrors)
      // el.append(
      //   $$(JATSImportDialog, { errors: this.state.importerErrors })
      // )
    }
    return el
  }

  _loadDocument() {
    const configurator = this.getConfigurator()
    this.props.buffer.readFile(this.props.documentPath).then((xmlString) => {
      let dom = DefaultDOMElement.parseXML(xmlString)
      dom = this.jatsImporter.import(dom)
      if (this.jatsImporter.hasErrored()) {
        console.error('Could not transform to TextureJATS')
        this.setState({
          importerErrors: this.jatsImporter.errors
        })
        return
      }
      const importer = configurator.createImporter('texture-jats')
      const doc = importer.importDocument(dom)
      window.doc = doc
      // create editor session
      const editorSession = new EditorSession(doc, {
        configurator: configurator
      })
      this.setState({
        editorSession: editorSession
      })
    })/*.catch((err) => {
      console.error(err)
      this.setState({
        loadingError: new Error('Loading failed')
      })
    })*/
  }

}
