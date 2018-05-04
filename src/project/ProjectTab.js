import { Component, FontAwesomeIcon } from 'substance'

export default class ProjectTab extends Component {

  getInitialState() {
    return {
      edit: false,
      error: false,
      menu: false
    }
  }

  render($$) {
    let el = $$('div').addClass('sc-project-tab')
    let entry = this.props.entry
    let docTypeEl = $$('div').addClass('se-doc-type')
    if (entry.type === 'article') {
      docTypeEl.append($$(FontAwesomeIcon, {icon: 'fa-align-left'}))
    } else if (entry.type === 'sheet') {
      docTypeEl.append($$(FontAwesomeIcon, {icon: 'fa-table'}))
    }
    el.append(docTypeEl)

    if (this.state.edit) {
      let input = $$('input').addClass('se-input').attr({value: entry.name})
          .ref('documentName')
          .on('blur', this._onBlur)
          .on('keydown', this._onKeyDown)
          .on('input', this._onInput)
      if (this.state.error) {
        input.addClass('sm-error')
      }
      el.addClass('sm-edit sm-active').append(input)
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
    let handled = false
    switch (e.key) {
      case 'Escape': {
        this._cancelEdit()
        handled = true
        break
      }
      case 'Enter': {
        this._validateAndConfirm(e)
        break
      }
      default:
        //
    }
    if (handled) {
      e.stopPropagation()
      e.preventDefault()
    }
  }

  _onBlur(e) {
    if (this._skipBlur || !this.isMounted()) {
      return
    }
    // only update if we are still editing
    if (this.state.edit) {
      this._validateAndConfirm(e)
    }
  }

  _onInput() {
    let err = this._validateNewName()
    if (err) {
      this.extendState({ error: err })
    } else if (this.state.error) {
      this.extendState({ error: false })
    }
  }

  _openDocument() {
    if (!this.isMounted()) return

    if (!this.props.active) {
      this.send('openDocument', this.props.entry.id)
    }
  }

  _removeDocument() {
    this.send('removeDocument', this.props.entry.id)
  }

  _toggleMenu(e) {
    e.preventDefault()
    e.stopPropagation()
    this.extendState({ menu: !this.state.menu })
  }

  _editDocumentName() {
    this.extendState({ edit: true, error: false })
    this._grabFocus()
  }

  _grabFocus() {
    let inputEl = this.refs.documentName.getNativeElement()
    inputEl.focus()
    if (inputEl.setSelectionRange) {
      let lastPos = this.refs.documentName.val().length
      inputEl.setSelectionRange(lastPos, lastPos)
    }
  }

  _cancelEdit() {
    this.refs.documentName.val(this.props.entry.name)
    this.extendState({ edit: false, error: false })
  }

  _validateAndConfirm(e) {
    const oldName = this.props.entry.name
    const newName = this.refs.documentName.val()
    let err = this._validateNewName()
    if (err) {
      e.stopPropagation()
      e.preventDefault()
      this._skipBlur = true
      this._alert(err)
      this.extendState({ edit:true, error: err })
      // HACK: the problem is that the input gets blurred
      // in a strange way when clicking the alert dialog button
      // it helps to wait a bit with re-activating the onBlur listener
      setTimeout(() => {
        this._skipBlur = false
      }, 100)
    } else {
      this.extendState({ edit: false })
      if (oldName !== newName) {
        this.send('updateDocumentName', this.props.entry.id, newName)
      }
    }
  }

  _validateNewName() {
    let newName = this.refs.documentName.val()
    newName = newName.trim()
    if (!/^[^']+$/.exec(newName)) {
      return "Name contains invalid characters!"
    }
    const archive = this.context.documentArchive
    let entries = archive.getDocumentEntries()
    for (let i = 0; i < entries.length; i++) {
      let entry = entries[i]
      if (entry.id === this.props.entry.id) continue
      if (entry.name === newName) {
        return "Another document with this name exists."
      }
    }
  }

  _alert(msg) {
    if (window.alert) {
      window.alert(msg) // eslint-disable-line no-alert
    }
  }

}
