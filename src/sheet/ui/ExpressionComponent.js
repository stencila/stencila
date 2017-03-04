import { Component } from 'substance'
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
    // Display cell teaser
    el.append($$(CellTeaserComponent, {node: node}))
    if (node.value !== undefined && node.displayMode !== 'cli') {
      el.append(
        $$('pre').append(node.value)
      )
    }
    return el
  }
}
