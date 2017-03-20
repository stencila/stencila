import {isArray, Component, Button} from 'substance'

export default
class CellStatusBar extends Component {

  getInitialState() {
    return {
      expandErrors: false
    }
  }

  didMount() {
    const cell = this.props.cell
    if (cell) {
      cell.on('evaluation:started', this.onEvaluationStarted, this)
      cell.on('evaluation:finished', this.onEvaluationFinished, this)
    }
  }

  dispose() {
    const cell = this.props.cell
    if (cell) {
      cell.off(this)
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
    // console.log('runtimeErrors', runtimeErrors)
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

  _renderRuntimeError($$, runtimeErrors) {
    let errorEl = $$('div').addClass('se-detailed-error')
    if (!isArray(runtimeErrors)) runtimeErrors = [runtimeErrors]
    runtimeErrors.forEach((runtimeError) => {
      if (errorEl.line >= 0) {
        errorEl.append(`Error in ${runtimeError.line}:`)
      }
      errorEl.append(runtimeError.message)
    })
    return errorEl
  }

  onToggleErrors() {
    this.extendState({
      expandErrors: !this.state.expandErrors
    })
  }

  onEvaluationStarted() {
    this.extendState({
      pending: true
    })
  }

  onEvaluationFinished() {
    this.extendState({
      pending: false
    })
  }

}