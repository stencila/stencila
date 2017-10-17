import { Component } from 'substance'

export default
class ObjectValueComponent extends Component {
  render($$) {
    let value = this.props.value
    let el = $$('div').addClass('sc-object-value')
    let ul = $$('ul')
    Object.keys(value.data).forEach((key) => {
      let json = JSON.stringify(value.data[key])
      if (json && json.length > 1000) json = json.slice(0, 1000)+'...'
      ul.append(
        $$('li').text(json)
      )
    })
    el.append(ul)
    return el
  }
}
