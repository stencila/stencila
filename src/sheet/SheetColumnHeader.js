import { NodeComponent, Tooltip } from 'substance'
import { getColumnLabel } from './sheetHelpers'
const DEFAULT_COLUMN_WIDTH = 100

class SheetColumnHeader extends NodeComponent {
  didMount() {
    super.didMount()
    const cell = this.props.node
    cell.on('issue:changed', this.rerender, this)
  }

  dispose() {
    super.dispose()
    const cell = this.props.node
    cell.off(this)
  }

  render($$) {
    const colIdx = this.props.colIdx
    const node = this.props.node
    const issueManager = this.context.issueManager

    let th = $$('th')
      .attr('data-col', colIdx)
      .addClass('sc-column-header')


    let cellIssues = issueManager.getColumnIssues(node.id)
    if(cellIssues.length > 0) {
      th.addClass('sm-issue sm-error')
    }

    let columnHeader = $$('div').addClass('se-column-title').append(
      $$('div').addClass('se-column-label').text(getColumnLabel(colIdx)),
      this.renderColumnName($$),
      this.renderColumnType($$)
    )

    th.append(
      columnHeader
    ).css({ width: this.getWidth() })

    return th
  }

  getWidth() {
    return this.props.node.attr('width') || DEFAULT_COLUMN_WIDTH
  }

  renderIcon($$, icon) {
    let iconEl = this.context.iconProvider.renderIcon($$, icon)
    return iconEl
  }

  renderColumnName($$) {
    const node = this.props.node
    let name = node.attr('name')
    if (!name) return

    let el = $$('div').addClass('se-column-name')
      .text(String(name))

    return el
  }

  renderColumnType($$) {
    // TODO: here we should discuss how to deal with units
    // we could introduce an extra type for different units
    // but IMO it is semantically more appropriate to have units
    // for number types, such as km, ms, MW
    // In that case we could rather display the unit than the type
    // 'km' instead of number
    // alternatively, we could introduce an extra row with the units
    const node = this.props.node
    let coltype = node.attr('type')

    if(!coltype || coltype === 'any') return

    let el = $$('div').addClass('se-column-type').append(
      this.renderIcon($$, coltype + '-cell-type'),
      $$(Tooltip, {
        text: this.getLabel(coltype)
      })
    )

    return el
  }
}

export default SheetColumnHeader
