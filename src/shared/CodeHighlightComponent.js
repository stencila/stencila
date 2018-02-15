import { Component } from 'substance'

export default
class CodeHighlightComponent extends Component {

  render($$) {
    const node = this.props.node
    let el = $$('span')
      .addClass('sc-code-highlight')
      .addClass('sm-'+node.name)
    el.append(this.props.children)
    return el
  }
}
