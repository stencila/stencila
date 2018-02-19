import { Component, FontAwesomeIcon } from 'substance'

export default class ProjectTab extends Component {

  getInitialState() {
    return {
      edit: false,
      menu: false
    }
  }

  render($$) {
    let el = $$('div').addClass('sc-project-tab')
    let entry = this.props.entry
    let docTypeEl = $$('div').addClass('se-doc-type')
    if (entry.type === 'application/jats4m') {
      docTypeEl.append($$(FontAwesomeIcon, {icon: 'fa-align-left'}))
    } else if (entry.type === 'application/sheetml') {
      docTypeEl.append($$(FontAwesomeIcon, {icon: 'fa-table'}))
    }
    el.append(docTypeEl)

    if (this.state.edit) {
      // Render input for document name editing
      el.addClass('sm-edit sm-active').append(
        $$('input').addClass('se-input').attr({value: entry.name})
          .ref('documentName')
          .on('blur', this._updateDocumentName)
          .on('keydown', this._onKeyDown)
      )
    } else {
      el.append(' '+(entry.name || entry.id))
        .on('click', this._openDocument)
        .on('dblclick', this._editDocumentName)
        .on('contextmenu', this._toggleMenu)

      if (this.props.active) {
        el.addClass('sm-active')
      }

      if (this.state.menu) {
        el.append(
          $$('ul').addClass('se-menu').append(
            $$('li').append('Remove')
              .on('click', this._removeDocument)
          )
        )
      }
    }

    // el.append(
    //   $$('div').addClass('se-toggle-menu').append(
    //     $$(FontAwesomeIcon, {icon: 'fa-caret-up'})
    //   ).on('click', this._toggleMenu.bind(this, entry.id))
    // )

    return el
  }

  _onKeyDown(e) {
    if (e.key === 'Enter') {
      // ATTENTION: It is important to trigger a blur event here, to ensure
      // that there is only one event source for updating the document name.
      this.refs.documentName.el.blur()
    }
  }

  _openDocument() {
    if (!this.props.active) {
      this.send('openDocument', this.props.entry.id)
    }
  }

  _editDocumentName() {
    this.extendState({edit: true})
    this.refs.documentName.el.focus()
  }

  _updateDocumentName() {
    const name = this.refs.documentName.val()
    this.extendState({ edit: false })
    this.send('updateDocumentName', this.props.entry.id, name)
  }

  _removeDocument() {
    this.send('removeDocument', this.props.entry.id)
  }

  _toggleMenu(e) {
    e.preventDefault()
    e.stopPropagation()
    this.extendState({ menu: !this.state.menu })
  }

}
