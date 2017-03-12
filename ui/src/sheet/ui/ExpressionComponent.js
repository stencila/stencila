import {Component, isNil} from 'substance'
import CellTeaserComponent from './CellTeaserComponent'

/**
  Displays expression cells, such that start with '=' and are
  not handled by a specific component.
*/
export default
class ExpressionComponent extends Component {
  render($$) {
    let node = this.props.node
    const el = $$('div').addClass('sc-expression')
    if (isNil(node.value)) {
      el.append($$(CellTeaserComponent, {node: node}))
    } else {
      // TODO: specific rendering for different value types
      el.append(node.value)
    }
    return el
  }
}
