import { Component, FontAwesomeIcon } from 'substance'
import documentTypes from '../documentTypes'

export default class ProjectTabs extends Component {

  render($$) {
    const da = this.props.documentArchive
    const documentEntries = da.getDocumentEntries()
    const nameEditor = this.state.nameEditor
    let el = $$('div').addClass('sc-project-tabs')

    documentEntries.forEach(entry => {
      if (_isVisible(entry)) {
        let tab = $$('div').addClass('se-tab').ref(entry.id)

        if (entry.id === nameEditor) {
          // Render input for document name editing
          tab.addClass('sm-edit').append(
            $$('input').addClass('se-input').attr({value: entry.name})
              .ref('documentName')
              .on('change', this._changeDocumentName.bind(this, entry.id))
          )
        } else {
          tab.append(entry.name || entry.id)
            .on('click', this._openDocument.bind(this, entry.id))
            .on('dblclick', this._editDocumentName.bind(this, entry.id))

          if (this.props.documentId === entry.id) {
            tab.addClass('sm-active')
          }
        }

        el.append(tab)
      }
    })

    let addDocumentButton = $$('button').append(
      $$(FontAwesomeIcon, {icon: 'fa-plus-circle'})
    ).on('click', this._addDocument)

    el.append(addDocumentButton)
    return el
  }

  _openDocument(documentId) {
    if(this.props.documentId !== documentId) {
      this.send('openDocument', documentId)
    }
  }

  _editDocumentName(documentId) {
    this.extendState({nameEditor: documentId})
  }

  _changeDocumentName(documentId) {
    const name = this.refs.documentName.val()
    this.send('editDocumentName', documentId, name)
    this.extendState({nameEditor: undefined})
  }

  _addDocument() {
    this.send('addDocument')
  }
}

function _isVisible(entry) {
  return Boolean(documentTypes[entry.type])
}
