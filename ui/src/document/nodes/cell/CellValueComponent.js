import { Component } from 'substance'

export default
class CellValueComponent extends Component {
  didMount() {
    const node = this.props.node
    if (node) {
      node.on('value:updated', this.rerender, this)
    }
  }
  dispose() {
    const node = this.props.node
    if (node) {
      node.off(this)
    }
  }
  render($$) {
    const node = this.props.node
    let el = $$('div').addClass('sc-cell-value')
    if (node.value) {
      el.append(
        $$('div').addClass('se-value')
          .text(String(node.valueType)+':'+String(node.value))
      )
    }
    if (node.errors && node.errors.length){
      node.errors.forEach((error) => {
        el.append(
          $$('div').addClass('se-error').text(String(error))
        )
      })
    }
    return el
  }
}