import { Tool } from 'substance'

/*
  Simplified version of SwitchTextTypeTool that looks nicer
  in overlay-only editing scenarios
*/
class MinimalSwitchTextTypeTool extends Tool {

  render($$) {
    let Button = this.getComponent('button')
    let el = $$('div').addClass('sc-minimal-switch-text-type')
    this.props.textTypes.forEach((textType) => {
      let button = $$(Button, {
        label: textType.name,
        active: this.props.currentTextType.name === textType.name,
        disabled: this.props.disabled,
        style: this.props.style
      }).attr('data-type', textType.name)
        .on('click', this.handleClick)
      el.append(button)
    })
    return el
  }

  handleClick(e) {
    let newTextType = e.currentTarget.dataset.type
    e.preventDefault()
    this.context.commandManager.executeCommand(this.getCommandName(), {
      textType: newTextType
    })
  }
}

export default MinimalSwitchTextTypeTool
