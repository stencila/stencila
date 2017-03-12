import { Component, TextPropertyComponent } from 'substance'

/**
  Displays constant cells, such that don't start with '='.

  Possible values of content are:

  '10'
  '10.5'
  'Hello world'
*/

export default
class ConstantComponent extends Component {
  render($$) {
    const node = this.props.node
    var el = $$('div').addClass('sc-constant')
    el.append(
      $$(TextPropertyComponent, {
        path: [node.id, 'content']
      })
    )
    return el
  }
}
