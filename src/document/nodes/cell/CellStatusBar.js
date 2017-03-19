import {Component, Button, forEach} from 'substance'

export default
class CellStatusBar extends Component {

  getInitialState() {
    return {
      expandErrors: false
    }
  }

  render($$) {
    const cell = this.props.cell
    let el = $$('div').addClass('sc-cell-status-bar')

    if (cell.hasRuntimeErrors()) {
      el.append(this.renderRuntimeErrors($$))
    }

    return el
  }

  renderRuntimeErrors($$) {
    let runtimeErrors = this.props.cell.getRuntimeErrors()
    let errorsEl = $$('div').addClass('se-errors')

    if (runtimeErrors.length > 1) {
      let toggleErrors = $$(Button, {
        label: this.state.expandErrors ? "\u25C0": "\u25BC",
        style: 'outline'
      }).ref('toggleErrors')
        // FIXME: it seems that we can not add classes anymore
        .addClass('se-toggle-errors')
        .on('click', this.onToggleErrors)
      errorsEl.append(toggleErrors)
    }

    if (this.state.expandErrors) {
      if (runtimeErrors.length > 0) {
        runtimeErrors.forEach((runtimeError) => {
          errorsEl.append(this._renderRuntimeError($$, runtimeError))
        })
      }
    } else {
      if (runtimeErrors.length > 1) {
        errorsEl.append($$('div').text('Multiple Errors'))
      } else {
        errorsEl.append(this._renderRuntimeError($$, runtimeErrors[0]))
      }
    }

    return errorsEl
  }

  _renderRuntimeError($$, runtimeError) {
    let errorEl = $$('div').addClass('se-detailed-error')
    forEach(runtimeError, (msg, line) => {
      errorEl.append(`Error in line ${line}:`).append(msg)
    })
    return errorEl
  }

  onToggleErrors() {
    this.extendState({
      expandErrors: !this.state.expandErrors
    })
  }

}