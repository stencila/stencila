import { Component } from 'substance'
import CellTeaserComponent from './CellTeaserComponent'

/**
  Displays a cell with value type 'image'.
 */
export default
class ImageCellComponent extends Component {
  render($$) {
    const node = this.props.node
    // Using .sc-sheet-image instead of .sc-image so we don't have style
    // clashes with native Substance Image
    const el = $$('div').addClass('sc-sheet-image')
    if (node.displayMode === 'cli') {
      el.append($$(CellTeaserComponent, {
        node: node,
        typeLabel: 'image'
      }))
    }
    if (node.value !== undefined && node.displayMode !== 'cli') {
      el.append(
        $$('img').attr('src', node.value)
      )
    }
    return el
  }
}
