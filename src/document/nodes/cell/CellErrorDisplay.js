import {isArray, Component} from 'substance'

export default
class CellErrorDisplay extends Component {

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
    let el = $$('div').addClass('sc-cell-error-display')
    if (cell.hasErrors()) {
      el.addClass('sm-has-errors')
      el.append(this.renderErrors($$))
    }
    return el
  }

  renderErrors($$) {
    let runtimeErrors = this.props.cell.getRuntimeErrors()
    let syntaxError = this.props.cell.getSyntaxError()
    let errorsEl = $$('div').addClass('se-errors')

    if (syntaxError) {
      errorsEl.append(
        $$('div').addClass('se-error').append('Syntax Error!')
      )
    }

    if (runtimeErrors.length > 0) {
      runtimeErrors.forEach((runtimeError) => {
        errorsEl.append(this._renderRuntimeError($$, runtimeError))
      })
    }

    return errorsEl
  }

  _renderRuntimeError($$, runtimeErrors) {
    let errorEl = $$('div').addClass('se-error')
    if (!isArray(runtimeErrors)) runtimeErrors = [runtimeErrors]
    runtimeErrors.forEach((runtimeError) => {
      if (errorEl.line >= 0) {
        errorEl.append(`Error in ${runtimeError.line}:`)
      }
      errorEl.append(runtimeError.message)
    })
    return errorEl
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
