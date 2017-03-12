import { Tool } from 'substance'

class ToggleInsertTool extends Tool {

  render($$) {
    let el

    if (this.state.open) {
      el = $$('div').addClass('se-tool')
      let toolGroups = this.context.toolGroups
      let tools = toolGroups.get('insert').tools
      let commandStates = this.context.commandManager

      tools.forEach((tool) => {
        let toolProps = Object.assign({}, commandStates[tool.name], {
          name: tool.name,
          label: tool.name,
          style: 'plain-dark',
          showIcon: true,
          showLabel: false
        })
        el.append(
          $$(tool.Class, toolProps)
        )
      })
    } else {
      el = super.render($$)
    }
    el.addClass('sc-toggle-insert')
    el.on('mousedown', this._onMousedown)
    return el
  }

  _onMousedown(e) {
    e.stopPropagation()
    e.preventDefault()
  }

  onClick(e) {
    e.preventDefault()
    e.stopPropagation()
    this.setState({
      open: true
    })
  }
}

export default ToggleInsertTool
