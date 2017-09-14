import { ToggleTool, Button, stop, Tooltip } from 'substance'

// Need this because we have dynamic labels
class CustomButton extends Button {
  renderLabel($$) {
    return $$('span').addClass('se-label').append(
      this.props.label
    )
  }
}

class Tool extends ToggleTool {

  render($$) {
    // TODO: use different class
    let el = $$('div')
      .addClass('sc-toggle-tool')
    el.addClass(this.getClassNames())
    el.append(
      this.renderButton($$)
    )
    el.append(
      this.renderTooltip($$)
    )
    el.on('mousedown', stop)
    return el
  }

  renderTooltip($$) {
    return $$(Tooltip, {
      text: this._getTooltipText()
    })
  }

}

class ContextMenuItem extends Tool {

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

  renderTooltip() {
    // Tooltips are a bit too noisy within a context menu
  }

  onClick(e) {
    e.preventDefault()
    e.stopPropagation()
    if (!this.props.disabled) {
      this.executeCommand()
    }
    this.el.emit('contextmenuitemclick')
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

export class SheetIssuesStatusTool extends Tool {

  render($$) {
    // TODO: use different class
    let el = $$('div')
      // .addClass('sc-toggle-tool')
      .addClass('sc-sheet-issues-status')
    const commandState = this.props.commandState
    if (commandState.severity) {
      el.addClass(`sm-${commandState.severity}`)
    }
    el.append(
      this.renderButton($$)
    )
    const numberOfIssues = commandState.numberOfIssues
    if (numberOfIssues) {
      el.append(
        $$('div').addClass('se-number-of-issues')
          // TODO: use this.getLabel()
          .text(`${numberOfIssues} problems`)
      )
    }

    el.on('mousedown', stop)
    return el
  }

  getClassNames() {
    let classNames = ['']
    return classNames.join(' ')
  }

  getIconName() {
    return 'sheet-issues'
  }
}
