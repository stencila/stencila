import Component from 'substance/ui/Component'

class InputComponent extends Component {

  render ($$) {
    let node = this.props.node
    let el = $$('input')
      .addClass('sc-input')
      .attr({
        name: node.name,
        type: node.type_
      })
      .on('change', event => {
        node.setValue(event.target.value)
        this.rerender() // to reset css width
      })
    if (node.value) el.attr('value', node.value)
    let width = null
    if (['text', 'number'].indexOf(node.type_) >= 0) {
      width = ((node.value ? node.value.length : 5) + 2) * 8 + 'px'
    }
    if (width) el.css('width', width)
    return el
  }

}

export default InputComponent
