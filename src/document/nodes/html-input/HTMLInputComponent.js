import { Component } from 'substance'

class HTMLInputComponent extends Component {

  render ($$) {
    let node = this.props.node
    let el = $$('span')
      .addClass('sc-html-input')
    if (this.props.isolatedNodeState) {
      el.addClass('sm-'+this.props.isolatedNodeState)
    }
    el.append(node.getValue())
    return el
  }

}

export default HTMLInputComponent
