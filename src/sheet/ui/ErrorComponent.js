import { Component} from 'substance'
import CellTeaserComponent from './CellTeaserComponent'

/**
  Displays a cell with value type 'error'.
 */
export default
class ErrorComponent extends Component {

  render($$) {
    const node = this.props.node
    const el = $$('div').addClass('sc-error')
      .addClass(node.getDisplayClass())
    // Display cell teaser
    el.append($$(CellTeaserComponent, {node: node}))
    el.append(
      $$('div').addClass('se-error-message').append(node.value)
    )
    return el
  }
}