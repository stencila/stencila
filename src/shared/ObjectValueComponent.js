import { Component } from 'substance'

export default
class ObjectValueComponent extends Component {
  render($$) {
    let value = this.props.value
    let el = $$('pre').addClass('sc-object-value')
    let json = JSON.stringify(value.data, null, ' ')
    if (json && json.length > 1000) json = json.slice(0, 1000) + '...'
    el.append(json)
    return el
  }
}
