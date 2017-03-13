import { Component } from 'substance'

class SelectComponent extends Component {

  render ($$) {
    let node = this.props.node
    let el = $$('span')
      .addClass('sc-select')
    if (this.props.isolatedNodeState) {
      el.addClass('sm-'+this.props.isolatedNodeState)
    }
    el.append(node.text)
    return el
  }

}

export default SelectComponent
