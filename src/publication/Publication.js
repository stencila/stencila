import { EditorSession, DefaultDOMElement, Component, forEach } from 'substance'
import { JATSImporter, JATSExporter, TextureConfigurator, EditorPackage as TextureEditorPackage } from 'substance-texture'
import PublicationManifest from './PublicationManifest'
import Engine from '../engine/Engine'
import FunctionManager from '../function/FunctionManager'
import DocumentEditorPackage from '../document/DocumentEditorPackage'
import DocumentEngineAdapter from '../document/DocumentEngineAdapter'

/*
  TODO: Implement full life cycle (like when other publication gets loaded)
*/
export default class Publication extends Component {

  constructor(parent, props) {
    super(parent, props)
    this.documentConfigurator = new TextureConfigurator()
    this.documentConfigurator.import(DocumentEditorPackage)
    this.jatsImporter = new JATSImporter()
    this.jatsExporter = new JATSExporter()

    // EXPERIMENTAL: stub implementation of a FunctionManager
    let functionManager = new FunctionManager()
    forEach(window.functions, (xml, name) => {
      functionManager.importFunction(name, xml)
    })
    this.engine = new Engine(props.host, functionManager)

  }

  getChildContext() {
    return {
      cellEngine: this.engine
    }
  }

  didMount() {
    this._loadPublication()
  }

  _getPublicationName() {
    return ''
  }

  render($$) {
    let el = $$('div').addClass('sc-publication')
    let activeDocument = this.state.activeDocument
    let buffer = this.state.buffer
    let manifest = this.state.manifest

    if (manifest) {
      el.append(
        $$('div').addClass('se-header').append(
          $$('input')
            .attr('type', 'text')
            .attr('value', this._getPublicationName())
            .attr('placeholder', 'Untitled Publication'),
          ...this._renderTabs($$),
          $$('button').append('+')
        )
      )
    }

    if (activeDocument && buffer) {
      if (activeDocument.type === 'document') {
        el.append(
          this._renderDocumentEditor($$)
        )
      } else if (activeDocument.type === 'sheet') {
        el.append(
          'TODO: Render sheet editor'
        )
      }
    }
    return el
  }

  _renderDocumentEditor($$) {
    // Loading error
    if (this.state.loadingError) {
      return $$('div').addClass('se-error').append(
        this.state.loadingError
      )
    }

    if (this.state.editorSession) {
      return $$(TextureEditorPackage.Editor, {
        editorSession: this.state.editorSession
      }).addClass('sc-document-editor')
    } else if (this.state.importerErrors) {
      console.error('JATS Import Error', this.state.importerErrors)
      // el.append(
      //   $$(JATSImportDialog, { errors: this.state.importerErrors })
      // )
    }
  }

  /*
    Tabs to select one document of the publication
  */
  _renderTabs($$) {
    let tabs = []
    let documents = this.state.manifest.getDocuments()
    documents.forEach((doc) => {
      let button = $$('button').append(doc.name)
        .on('click', this._openDocument.bind(this, doc))
      if (this.state.activeDocument.path === doc.path) {
        button.addClass('sm-active')
      }
      tabs.push(button)
    })
    return tabs
  }

  _openDocument(doc) {
    this.extendState({
      activeDocument: doc
    })
  }

  getBackend() {
    return this.props.backend
  }

  _loadPublication() {
    if (this.props.publicationId) {
      let backend = this.getBackend()
      let buffer
      backend.getBuffer(this.props.publicationId).then((buf) => {
        buffer = buf
        return buffer.readFile('manifest.xml')
      }).then((manifestXML) => {
        let manifest = new PublicationManifest(manifestXML)
        let activeDocument = manifest.getDocuments()[0]

        // TODO: we need to load all documents+sheets on start
        this._loadDocument(activeDocument, buffer).then((editorSession) => {
          // WIP: connecting the document instance with
          // the cell engine
          let engineAdapter = new DocumentEngineAdapter(editorSession)
          engineAdapter.connect(this.engine)
          // TODO: we need to load all resources (docs and assets)
          // as there might be cross references
          // the engine should be updated once all resources have been
          // registered
          this.engine.update()

          this.setState({
            buffer,
            manifest,
            activeDocument,
            editorSession
          })
        })
      })
    }
  }

  // TODO: we need to check for documentRecord.type (sheet|document)
  _loadDocument(documentRecord, buffer) {
    return new Promise((resolve, reject) => {
      buffer.readFile(documentRecord.path).then((xmlString) => {
        let editorSession
        if (documentRecord.type === 'document') {
          editorSession = this._importDocument(xmlString)
        } else {
          reject(new Error('Unsupported documentType '+ documentRecord.type))
        }
        resolve(editorSession)
      })
    })

  }

  _importDocument(xmlString) {
    let dom = DefaultDOMElement.parseXML(xmlString)
    dom = this.jatsImporter.import(dom)
    if (this.jatsImporter.hasErrored()) {
      // TODO: inspect this.jatsImporter.errors
      throw new Error('Could not transform to TextureJATS')
    }
    const importer = this.documentConfigurator.createImporter('texture-jats')
    const doc = importer.importDocument(dom)
    window.doc = doc
    // create editor session
    return new EditorSession(doc, {
      configurator: this.documentConfigurator
    })
  }

}
