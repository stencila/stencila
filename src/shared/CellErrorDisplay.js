import { Component } from 'substance'
import { getCellState } from './cellHelpers'

export default
class CellErrorDisplay extends Component {

  render($$) {
    const cell = this.props.cell
    const cellState = getCellState(cell)
    let el = $$('div').addClass('sc-cell-error-display')
    if (cellState && cellState.hasErrors()) {
      el.addClass('sm-has-errors')
      el.append(this.renderErrors($$))
    }
    return el
  }

  renderErrors($$) {
    const cell = this.props.cell
    const cellState = getCellState(cell)

    let errorsEl = $$('div').addClass('se-errors')
    if (cellState.hasErrors()) {
      cellState.errors.forEach(err => {
        errorsEl.append(
          $$('div').addClass('se-error').append(
            'Error: ',
            err.message
          )
        )
      })
    }
    return errorsEl
  }

}
