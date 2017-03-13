import { Component } from 'substance'

class RangeInputComponent extends Component {

  render ($$) {
    let node = this.props.node
    let el = $$('span')
      .addClass('sc-range-input')
    if (this.props.isolatedNodeState) {
      el.addClass('sm-'+this.props.isolatedNodeState)
    }
    el.append(node.value)
    return el
  }

}

export default RangeInputComponent
