import { Component } from 'substance'

class OutputComponent extends Component {

  didMount () {
    this.props.node.on('content:changed', this.rerender, this)
  }

  render ($$) {
    let node = this.props.node
    let el = $$('output')
      .addClass('sc-output')
      .html(node.content)
    return el
  }

}

export default OutputComponent
