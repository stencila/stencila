import { Component } from 'substance'

class IncludeComponent extends Component {

  render ($$) {
    let node = this.props.node
    let el = $$('div')
      .addClass('sc-include')
    if (node.content) {
      el.append(
        $$('div')
          .addClass('se-content')
          .html(
            node.content
          )
      )
    }
    return el
  }

  didMount () {
    this.props.node.on('changed', () => {
      this.rerender()
    })
  }

}

export default IncludeComponent
