import { Tool, Component } from 'substance'

class EditSelectTool extends Tool {

  render($$) {
    let InputSettingsBar = this.getComponent('input-settings-bar')
    let el = $$('div').addClass('sc-edit-select-tool')

    el.append(
      $$(InputSettingsBar, this.props)
        .on('toggle', this._onToggleSettings)
    )

    if (this.state.showSettings) {
      el.append($$(Settings, this.props))
    } else {
      el.append($$(Options, this.props))
    }

    // Render settings
    return el
  }

  _onToggleSettings() {
    this.setState({
      showSettings: !this.state.showSettings
    })
  }
}

class Options extends Component {
  render($$) {
    let node = this.props.node
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
    return optionsEl
  }

  _onOptionClicked(selectedIndex) {
    this.context.editorSession.transaction((tx) => {
      tx.set([this.props.node.id, 'selectedIndex'], selectedIndex)
    })
    this.rerender()
  }
}

class Settings extends Component {
  render($$) {
    return $$('div').append('TODO: Implement')
  }
}

export default EditSelectTool
