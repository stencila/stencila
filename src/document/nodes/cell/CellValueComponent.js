import { Component, isNil } from 'substance'

export default
class CellValueComponent extends Component {
  didMount() {
    const cell = this.props.cell
    if (cell) {
      cell.on('value:updated', this.rerender, this)
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
    let el = $$('div').addClass('sc-cell-value')
    // EXPERIMENTAL: caching the value data so that
    // we can render something while the engine is updating
    // still, not sure yet if this is the right place to do
    let value, valueType
    // TODO: Eventually, I want to have a state on the cell itself
    // which is managed by the engine:
    let pending = false
    if (!isNil(cell.value)) {
      value = cell.value
      valueType = cell.valueType
    } else if (!isNil(this._lastValidValue)) {
      value = this._lastValidValue
      valueType = this._lastValidValueType
      pending = true
    }
    if (!isNil(value)) {
      const registry = this.context.componentRegistry
      let ValueDisplay = registry.get('value:'+valueType)
      if (ValueDisplay) {
        el.append(
          $$(ValueDisplay, {value}).ref('value')
        )
      } else {
        let valueStr = String(value)
        if (valueStr.length > 10000) {
          valueStr = valueStr.slice(0, 10000)+'...'
        }
        el.append(
          $$('div').addClass('se-value').append(
            String(valueType), ':', valueStr
          )
        )
      }
    }
    if (pending) el.addClass('sm-pending')
    return el
  }
}