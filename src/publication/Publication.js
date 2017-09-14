import { Component } from 'substance'
import DocumentPage from '../document/DocumentPage'
import PublicationManifest from './PublicationManifest'

/*
  TODO: Implement full life cycle (like when other publication gets loaded)
*/
export default class Publication extends Component {

  didMount() {
    this._loadBufferAndManifest()
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

  _loadBufferAndManifest() {
    if (this.props.publicationId) {
      let backend = this.getBackend()
      let buffer
      backend.getBuffer(this.props.publicationId).then((buf) => {
        buffer = buf
        return buffer.readFile('manifest.xml')
      }).then((manifestXML) => {
        let manifest = new PublicationManifest(manifestXML)
        let activeDocument = manifest.getDocuments()[0]
        this.setState({
          buffer,
          manifest,
          activeDocument
        })
      })
    }
  }

}
