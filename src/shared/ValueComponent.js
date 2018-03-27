import { Component } from 'substance'

export default
class ValueComponent extends Component {

  render($$) {
    const registry = this.context.componentRegistry
    let el = $$('div').addClass('sc-cell-value')
    // TODO: we want to treat values like Promises
    // to support complexer things, such as pointer types, etc.
    let valueType = this.props.type
    let ValueDisplay = registry.get('value:'+valueType)
    if (ValueDisplay) {
      let valueEl = $$(ValueDisplay, {value: this.props}).ref('value')
      el.append(valueEl)
    }
    return el
  }

}
