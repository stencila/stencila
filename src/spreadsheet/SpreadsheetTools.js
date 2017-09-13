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
    let n = commandState.nrows || 1
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

export class DeleteRowsTool extends ContextMenuItem {
  getButtonLabel() {
    let commandState = this.props.commandState
    let label
    if (commandState.nrows > 1) {
      label = this.getLabel('delete-rows')
        .replace('${startRow}', commandState.startRow)
        .replace('${endRow}', commandState.endRow)
    } else {
      label = this.getLabel('delete-row')
    }
    return label
  }
}

class InsertColumnsTool extends ContextMenuItem {
  getButtonLabel() {
    let commandState = this.props.commandState
    let n = commandState.ncolumns || 1
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

export class DeleteColumnsTool extends ContextMenuItem {
  getButtonLabel() {
    let commandState = this.props.commandState
    let label
    if (commandState.ncolumns > 1) {
      label = this.getLabel('delete-columns')
        .replace('${startCol}', commandState.startCol)
        .replace('${endCol}', commandState.endCol)
    } else {
      label = this.getLabel('delete-column')
    }
    return label
  }
}

export class OpenColumnSettingsTool extends ContextMenuItem {
  getButtonLabel() {
    return this.getLabel('open-column-settings')
  }
}
