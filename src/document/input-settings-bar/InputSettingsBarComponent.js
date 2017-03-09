import { Component } from 'substance'

export default class InputSettingsBar extends Component {

  render($$) {
    let Button = this.getComponent('button')
    let node = this.props.node
    let el = $$('div').addClass('sc-input-settings-bar')

    el.append(
      $$('input')
        .addClass('st-tiny-input')
        .attr('placeholder', 'Enter variable name')
        .val(node.name)
        .on('change', this._onNameChanged)
        .ref('nameInput'),
      $$(Button, {
        icon: 'toggle-settings',
        style: 'plain-dark'
      }).on('click', this._onToggleSettings)
    )
    return el
  }

  _onNameChanged() {
    let name = this.refs.nameInput.val()
    this.context.editorSession.transaction((tx) => {
      tx.set([this.props.node.id, 'name'], name)
    })
    this.rerender()
  }

  _onToggleSettings() {
    this.el.emit('toggle')
  }

}
