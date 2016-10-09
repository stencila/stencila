import Component from 'substance/ui/Component'

class PrintComponent extends Component {

  didMount () {
    this.props.node.on('content:changed', this.rerender, this)
  }

  dispose () {
    this.props.node.off(this)
  }

  render ($$) {
    var node = this.props.node
    return $$('span')
      .addClass('sc-print' + (node.error ? ' sm-error' : ''))
      .append(node.content.length ? node.content : ' ')
  }

}

export default PrintComponent
