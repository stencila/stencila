import { Component } from 'substance'

/**
  Displays constant cells, such that don't start with '='.

  Possible values of content are:

  '10'
  '10.5'
  'Hello world'
  'Hello <strong>world</strong>'
*/

export default
class ConstantComponent extends Component {
  render($$) {
    var el = $$('div').addClass('sc-constant')
    el.append(this.props.node.content)
    return el
  }
}
