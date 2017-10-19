import { Component } from 'substance'

export default
class CellValueComponent extends Component {

  render($$) {
    const registry = this.context.componentRegistry
    const cell = this.props.cell
    const cellState = cell.state
    let el = $$('div').addClass('sc-cell-value')
    // console.log('rendering %s -- value = ', cell.id, value)
    if (cellState && cellState.hasValue()) {
      // TODO: we want to treat values like Promises
      // to support complexer things, such as pointer types, etc.
      let value = cellState.getValue()
      let valueType = value.getType()
      let ValueDisplay = registry.get('value:'+valueType)
      if (ValueDisplay) {
        let valueEl = $$(ValueDisplay, {value, cell}).ref('value')
        if (cellState.hasErrors()) {
          valueEl.addClass('sm-has-errors')
        }
        el.append(valueEl)
      } else {
        let valueStr = value.toString()
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

}
