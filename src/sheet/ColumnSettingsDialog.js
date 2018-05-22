import { Component, getRelativeBoundingRect, domHelpers, isEqual } from 'substance'

export default class ColumnSettingsDialog extends Component {

  didMount() {
    this._position()
  }

  render($$) {
    let el = $$('div')
      .addClass('sc-dialog')
      .addClass('sc-column-settings-dialog')
    el.append(this._renderHead($$))
      .append(this._renderBody($$))
      .append(this._renderFoot($$))
      .addClass('sm-hidden')
      .on('mousedown', domHelpers.stop)
      .on('keydown', this._onKeyDown)
    return el
  }

  _renderHead($$) {
    let head = $$('div').addClass('se-head')
    let title = $$('div').addClass('se-title').text(this.getTitle())
    head.append(title)
    return head
  }

  _renderBody($$) {
    const node = this._getNode()
    // const type = node.attr('type')
    let body = $$('div').addClass('se-body')
    body.append($$('div').addClass('se-item').append(
      $$('div').addClass('se-label').text(this.getLabel('name')),
      $$('input').ref('name')
        .addClass('se-input sm-name')
        .attr('type', 'text')
        .val(node.attr('name'))
    ))

    // TODO: Bring back typed cells
    // let typeSelect = $$('select').ref('type')
    //   .addClass('se-input sm-type')
    //   .val(node.attr('type'))
    // // TODO: get types from schema
    // ;['any', 'number', 'integer', 'string', 'boolean'].forEach((t) => {
    //   let option = $$('option')
    //     .attr('value', t)
    //     .text(this.getLabel(t))
    //   if (t === type) {
    //     option.attr('selected', true)
    //   }
    //   typeSelect.append(option)
    // })
    // body.append($$('div').addClass('se-item').append(
    //   $$('div').addClass('se-label').text(this.getLabel('type')),
    //   typeSelect
    // ))

    return body
  }

  _renderFoot($$) {
    let foot = $$('div').addClass('se-foot')
    foot.append(
      $$('button').addClass('se-confirm').text(this.getLabel('ok'))
        .on('click', this._onConfirm)
    )
    foot.append(
      $$('button').addClass('se-cancel').text(this.getLabel('cancel'))
        .on('click', this._onCancel)
    )
    return foot
  }

  getTitle() {
    return this.getLabel('title:column-settings')
  }

  _position() {
    let sheetComponent = this._getSheetComponent()
    let cellComponent = this._getCellComponent()
    if (cellComponent) {
      let rect = getRelativeBoundingRect(cellComponent.el, sheetComponent.el)
      this.el.css({
        top: rect.top,
        left: rect.left
      })
      this.el.removeClass('sm-hidden')
    }
  }

  _getSheetComponent() {
    return this.props.params.surface
  }

  _getCommandState() {
    return this.props.params.commandState
  }

  _getCellComponent() {
    let commandState = this._getCommandState()
    let sheetComponent = this._getSheetComponent()
    return sheetComponent._getCellComponent(-1, commandState.colIdx)
  }

  _getNode() {
    let commandState = this._getCommandState()
    return commandState.node
  }

  _getEditorSession() {
    return this.props.params.editorSession
  }

  _hide() {
    this._getSheetComponent()._hideDialog()
  }

  _onConfirm() {
    // hide the dialog
    this._hide()
    // and update the model
    const node = this._getNode()
    let oldAttr = {
      name: node.attr('name'),
      // type: node.attr('type')
    }
    let newAttr = {
      name: this.refs.name.val(),
      // type: this.refs.type.val()
    }
    if (!isEqual(oldAttr, newAttr)) {
      let editorSession = this._getEditorSession()
      let nodeId = node.id
      editorSession.transaction((tx) => {
        let node = tx.get(nodeId)
        node.attr(newAttr)
      })
    }
  }

  _onCancel() {
    this._hide()
  }

  _onKeyDown(e) {
    if(e.keyCode === 13) {
      this._onConfirm()
    } else if (e.keyCode === 27) {
      this._hide()
    }
  }

}
