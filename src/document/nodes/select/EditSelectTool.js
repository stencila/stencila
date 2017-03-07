import { Tool } from 'substance'

class EditSelectTool extends Tool {

  render($$) {
    let Button = this.getComponent('button')
    let node = this.props.node
    let el = $$('div').addClass('sc-edit-select-tool')

    // Render settings
    let settingsEl = $$('div').addClass('se-settings')
    settingsEl.append(
      $$('input')
        .ref('name')
        .attr('placeholder', 'Enter variable name')
        .val(node.name),
      $$(Button, {
        icon: 'select-settings',
        style: 'plain-dark'
      })
    )
    el.append(settingsEl)

    // Render options
    let optionsEl = $$('div').addClass('se-options')
    node.options.forEach((option, optionIndex) => {
      let optionEl = $$('button').append(
        $$('div').addClass('se-text').append(option.text),
        $$('div').addClass('se-value').append(option.value)
      )
      .addClass('se-option')
      .on('click', this._onOptionClicked.bind(this, optionIndex))
      if (optionIndex === node.selectedIndex) {
        optionEl.addClass('sm-selected')
      }
      optionsEl.append(optionEl)
    })
    el.append(optionsEl)

    return el
  }

  _onOptionClicked(selectedIndex) {
    this.context.editorSession.transaction((tx) => {
      tx.set([this.props.node.id, 'selectedIndex'], selectedIndex)
    })
    this.rerender()
  }
}

export default EditSelectTool
