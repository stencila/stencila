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
      })
    if (node.value) el.attr('value', node.value)
    return el
  }

}

export default InputComponent
