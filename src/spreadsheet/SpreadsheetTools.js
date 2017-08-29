import { Tool, Button } from 'substance'

// Need this because we have dynamic labels
class CustomButton extends Button {
  renderLabel($$) {
    return $$('span').addClass('se-label').append(
      this.props.label
    )
  }
}

function _stopPropagation(e) {
  e.stopPropagation()
}

class ContextMenuItem extends Tool {

  render($$) {
    return super.render($$).on('mousedown', _stopPropagation)
  }

  renderButton($$) {
    let commandState = this.props.commandState
    let btnProps = {
      label: this.getButtonLabel(),
      active: commandState.active,
      disabled: commandState.disabled,
      theme: this.props.theme
    }
    let btn = $$(CustomButton, btnProps).on('click', this.onClick)
    return btn
  }

  onClick(e) {
    e.preventDefault()
    e.stopPropagation()
    if (!this.props.disabled) {
      this.executeCommand()
    }
    this.el.emit('contextmenuitemclick')
  }

  _getTooltipText() {
    return this.getButtonLabel()
  }

}

class InsertRowsTool extends ContextMenuItem {

  getButtonLabel() {
    let commandState = this.props.commandState
    let n = commandState.rows || 1
    let pattern = this.getLabel(this.labelKey)
    let label = pattern.replace('${n}', n)
    return label
  }

}

export class InsertRowsAboveTool extends InsertRowsTool {
  get labelKey() {
    return 'insert-rows-above'
  }
}

export class InsertRowsBelowTool extends InsertRowsTool {
  get labelKey() {
    return 'insert-rows-below'
  }
}

class InsertColumnsTool extends ContextMenuItem {

  getButtonLabel() {
    let commandState = this.props.commandState
    let n = commandState.columns || 1
    let pattern = this.getLabel(this.labelKey)
    let label = pattern.replace('${n}', n)
    return label
  }

}

export class InsertColumnsLeftTool extends InsertColumnsTool {
  get labelKey() {
    return 'insert-columns-left'
  }
}

export class InsertColumnsRightTool extends InsertColumnsTool {
  get labelKey() {
    return 'insert-columns-right'
  }
}
