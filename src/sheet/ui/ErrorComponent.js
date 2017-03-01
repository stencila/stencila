import { Component} from 'substance'
import CellTeaserComponent from './CellTeaserComponent'

export default
class ErrorComponent extends Component {

  render($$) {
    var node = this.props.node
    var el = $$('div').addClass('sc-error')

    el.addClass(node.getDisplayClass())

    // Display cell teaser
    el.append($$(CellTeaserComponent, {node: node}))

    el.append(
      $$('div').addClass('se-error-message').append(node.value)
    )
    return el
  }
}