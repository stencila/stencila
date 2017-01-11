import Component from 'substance/ui/Component'

class SelectComponent extends Component {

  render ($$) {
    let node = this.props.node
    let el = $$('select')
      .addClass('sc-select')
      .attr('name', node.name)
      .on('change', event => {
        node.setValue(event.target.value)
      })
    for (let details of node.options) {
      let parts = details.split('\t')
      let option = $$('option')
          .attr('value', parts[0])
          .text(parts[1])
      if (parts[0] === node.value) {
        option.attr('selected', true)
      }
      el.append(option)
    }
    return el
  }

}

export default SelectComponent
