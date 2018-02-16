import { Component, FontAwesomeIcon } from 'substance'
import documentTypes from '../documentTypes'

export default class ProjectTabs extends Component {

  render($$) {
    const da = this.props.documentArchive
    const documentEntries = da.getDocumentEntries()
    const nameEditor = this.state.nameEditor
    const menu = this.state.menu
    let el = $$('div').addClass('sc-project-tabs')

    documentEntries.forEach(entry => {
      if (_isVisible(entry)) {
        let tab = $$('div').addClass('se-tab').ref(entry.id)
          .on('mousedown', this._activateTabDrag.bind(this, entry.id))
          .on('dragend', this._onDragend.bind(this, entry.id))
          .on('dragover', this._onDragOver.bind(this, entry.id))
          .on('dragleave', this._onDragLeave.bind(this, entry.id))
          .on('dragstart', this._onDrag)
          .on('dragenter', this._onDrag)

        if (entry.id === nameEditor) {
          // Render input for document name editing
          tab.addClass('sm-edit sm-active').append(
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

          if (entry.id === menu) {
            tab.append(
              $$('ul').addClass('se-menu').append(
                $$('li').append('Remove')
                  .on('click', this._removeDocument.bind(this, entry.id))
              )
            )
          }
        }

        tab.append(
          $$('div').addClass('se-toggle-menu').append(
            $$(FontAwesomeIcon, {icon: 'fa-caret-up'})
          ).on('click', this._toggleMenu.bind(this, entry.id))
        )

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

  _removeDocument(documentId, e) {
    e.preventDefault()
    e.stopPropagation()
    this.send('removeDocument', documentId)
    this.extendState({menu: undefined})
  }

  _toggleMenu(documentId, e) {
    e.preventDefault()
    e.stopPropagation()
    const active = this.state.menu
    const menu = active === documentId ? undefined : documentId
    this.extendState({menu: menu})
  }

  _activateTabDrag(entityId) {
    let tab = this.refs[entityId]
    tab.attr('draggable', true)
  }

  _deactivateTabDrag(entityId) {
    let tab = this.refs[entityId]
    tab.attr('draggable', false)
  }

  _onDragend(entityId) {
    this._deactivateTabDrag(entityId)
    this._onReorder(entityId, this.currentTarget)
    delete this.currentTarget
    delete this.currentVisualTarget
  }

  _onDragOver(entityId) {
    if(this.currentVisualTarget !== entityId) {
      this.currentTarget = entityId
      this.currentVisualTarget = entityId
      let tab = this.refs[entityId]
      tab.addClass('sm-drop')
    }
  }

  _onDragLeave(entityId) {
    delete this.currentVisualTarget
    let tab = this.refs[entityId]
    tab.removeClass('sm-drop')
  }

  _onDrag(e) {
    // Stop event propagation for the dragstart and dragenter
    // events, to avoid editor drag manager errors
    e.stopPropagation()
  }

  _onReorder(documentId, target) {
    const da = this.props.documentArchive
    const documentEntries = da.getDocumentEntries()
    let entriesIds = documentEntries.map(entry => { return entry.id })
    const currentPos = entriesIds.indexOf(documentId)
    const targetPos = entriesIds.indexOf(target)
    entriesIds[currentPos] = target
    entriesIds[targetPos] = documentId
    this.send('editDocumentOrder', entriesIds)
  }
}

function _isVisible(entry) {
  return Boolean(documentTypes[entry.type])
}
