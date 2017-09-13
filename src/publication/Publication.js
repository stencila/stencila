import { Component, DefaultDOMElement } from 'substance'
import DocumentPage from '../document/DocumentPage'

/*
  TODO: Implement full life cycle (like when other publication gets loaded)
*/
export default class Publication extends Component {

  didMount() {
    this._loadBufferAndManifest()
  }

  render($$) {
    let el = $$('div').addClass('sc-publication')
    let activeDocument = this.state.activeDocument
    let buffer = this.state.buffer
    if (activeDocument) {
      if (activeDocument.type === 'document') {
        el.append(
          $$(DocumentPage, {
            buffer,
            documentPath: activeDocument.path
          })
        )
      } else if (activeDocument.type === 'sheet') {
        el.append(
          'TODO: Render sheet editor'
        )
      }
    }
    return el
  }

  getBackend() {
    return this.props.backend
  }

  // TODO: Create abstraction class for manifest access and manipulation
  _loadBufferAndManifest() {
    if (this.props.publicationId) {
      let backend = this.getBackend()
      let buffer
      backend.getBuffer(this.props.publicationId).then((buf) => {
        buffer = buf
        return buffer.readFile('manifest.xml')
      }).then((manifestXML) => {
        let manifest = this._parseManifest(manifestXML)
        // The first document in the publication is the activeDoc
        let firstDoc = manifest.find('document')
        this.setState({
          buffer: buffer,
          manifest: manifest,
          activeDocument: {
            path: firstDoc.attr('src'),
            type: firstDoc.attr('type')
          }
        })
      })
    }
  }

  _parseManifest(manifestXML) {
    return DefaultDOMElement.parseXML(manifestXML)
  }

}
