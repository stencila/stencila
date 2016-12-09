import Component from 'substance/ui/Component'

class InputComponent extends Component {

  render ($$) {
    let node = this.props.node
    return $$('input')
      .addClass('sc-input')
      .attr({
        name: node.name,
        type: node.displayType
      })
      .on('change', event => {
        node.setValue(event.target.value)
      })
  }

}

export default InputComponent
