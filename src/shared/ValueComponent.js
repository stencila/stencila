import { Component } from 'substance'

export default
class ValueComponent extends Component {

  render($$) {
    const registry = this.context.componentRegistry
    let el = $$('div').addClass('sc-cell-value')
    // TODO: we want to treat values like Promises
    // to support complexer things, such as pointer types, etc.
    let value = this.props.data
    let valueType = this.props.type
    let ValueDisplay = registry.get('value:'+valueType)
    if (ValueDisplay) {
      let valueEl = $$(ValueDisplay, {value: this.props}).ref('value')
      // if (cellState.hasErrors()) {
      //   valueEl.addClass('sm-has-errors')
      // }
      el.append(valueEl)
    } else {
      console.log(this.props)
      // let valueStr = value.toString()
      // if (valueStr && valueStr.length > 10000) {
      //   valueStr = valueStr.slice(0, 10000)+'...'
      // }
      el.append(
        $$('div').addClass('se-default-value').append('')
      )
    }
    return el
  }

}
