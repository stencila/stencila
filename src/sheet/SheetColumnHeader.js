import { NodeComponent } from 'substance'
import { getColumnLabel } from './sheetHelpers'
const DEFAULT_COLUMN_WIDTH = 100

class SheetColumnHeader extends NodeComponent {

  render($$) {
    const colIdx = this.props.colIdx
    const node = this.props.node
    let th = $$('th')
      .attr('data-col', colIdx)
      .addClass('sc-column-header')
    th.append(
      $$('div').addClass('se-column-label').text(getColumnLabel(colIdx))
    )
    let name = node.attr('name') || "\u00A0"
    th.append(
      $$('div').addClass('se-column-name').text(String(name))
    )
    // TODO: here we should discuss how to deal with units
    // we could introduce an extra type for different units
    // but IMO it is semantically more appropriate to have units
    // for number types, such as km, ms, MW
    // In that case we could rather display the unit than the type
    // 'km' instead of number
    // alternatively, we could introduce an extra row with the units
    let coltype = node.attr('type') || "\u00A0"
    th.append(
      $$('div').addClass('se-column-type').text(this.getLabel(coltype))
    )
    th.css({ width: this.getWidth() })
    return th
  }

  getWidth() {
    return this.props.node.attr('width') || DEFAULT_COLUMN_WIDTH
  }
}

export default SheetColumnHeader
