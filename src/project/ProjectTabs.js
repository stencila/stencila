import { Component, FontAwesomeIcon } from 'substance'
import documentTypes from '../documentTypes'

export default class ProjectTabs extends Component {

  render($$) {
    let el = $$('div').addClass('sc-project-tabs')
    let da = this.props.documentArchive
    let documentEntries = da.getDocumentEntries()
    documentEntries.forEach((entry) => {
      if (_isVisible(entry)) {
        let button = $$('button').append(entry.name || entry.id)
          .on('click', this._openDocument.bind(this, entry.id))
        if (this.props.documentId === entry.id) {
          button.addClass('sm-active')
        }
        el.append(button)
      }
    })
    // Create new button
    let button = $$('button').append(
      $$(FontAwesomeIcon, {icon: 'fa-plus-circle'})
    )
      .on('click', this._addDocument)
    el.append(button)
    return el
  }

  _openDocument(documentId) {
    if(this.props.documentId !== documentId) {
      this.send('openDocument', documentId)
    }
  }

  _addDocument() {
    this.send('addDocument')
  }
}

function _isVisible(entry) {
  return Boolean(documentTypes[entry.type])
}
