import { Component, isNil } from 'substance'

export default
class CellValueComponent extends Component {

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
    const registry = this.context.componentRegistry
    const engine = this.context.cellEngine
    const cell = this.props.cell
    let el = $$('div').addClass('sc-cell-value')
    let value = engine.getValue(cell.id)
    console.log('rendering %s -- value = ', cell.id, value)
    if (!isNil(value)) {
      let valueType = engine.getValueType(value)
      let ValueDisplay = registry.get('value:'+valueType)
      if (ValueDisplay) {
        let valueEl = $$(ValueDisplay, {value, cell}).ref('value')
        if (engine.hasErrors(cell.id)) {
          valueEl.addClass('sm-has-errors')
        }
        el.append(valueEl)
      } else {
        let valueStr = JSON.stringify(value)
        if (valueStr && valueStr.length > 10000) {
          valueStr = valueStr.slice(0, 10000)+'...'
        }
        el.append(
          $$('div').addClass('se-default-value').append(valueStr)
        )
      }
    }
    return el
  }

  onEvaluationStarted() {
    // console.log('### CellValueComponent.onEvaluationStarted')
    this.el.addClass('sm-pending')
  }

  onEvaluationFinished() {
    // console.log('### CellValueComponent.onEvaluationFinished')
    this.el.removeClass('sm-pending')
    this.rerender()
  }
}
