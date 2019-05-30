import { DefaultDOMElement, NodeComponent, Tooltip } from 'substance'
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

    let th = $$('th')
      .attr('data-col', colIdx)
      .addClass('sc-column-header')

    let columnHeader = $$('div').addClass('se-column-title').append(
      $$('div').addClass('se-column-label').text(getColumnLabel(colIdx)),
      this.renderColumnName($$),
      this.renderColumnType($$)
    )

    th.append(
      columnHeader,
      $$('div').addClass('se-resize-handle')
        .on('mousedown', this._onMouseDown)
    ).css({ width: this.getWidth() }).ref('header')

    return th
  }

  getWidth() {
    // HACK: because XML importer does not convert to the right type
    // we need to do it here
    return Number.parseInt(this.props.node.attr('width'),10) || DEFAULT_COLUMN_WIDTH
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

  _onMouseDown(e) {
    e.preventDefault()
    e.stopPropagation()

    this._mouseDown = true
    this._startX = e.pageX
    this._colWidth = this.refs.header.getWidth()
    let _window = DefaultDOMElement.getBrowserWindow()
    _window.on('mousemove', this._onMouseMove, this)
    _window.on('mouseup', this._onMouseUp, this)
  }

  _onMouseMove(e) {
    if (this._mouseDown) {
      const width = this._colWidth + (e.pageX - this._startX)
      this.refs.header.css({ width: width })
      const editor = this.context.editor
      editor.refs.sheet._positionSelection()
    }
  }

  _onMouseUp(e) {
    this._mouseDown = false
    let _window = DefaultDOMElement.getBrowserWindow()
    _window.off('mousemove', this._onMouseMove, this)
    _window.off('mouseup', this._onMouseUp, this)

    const node = this.props.node
    const nodeId = node.id
    const width = this._colWidth + (e.pageX - this._startX)
    const editorSession = this.context.editorSession
    editorSession.transaction((tx) => {
      let node = tx.get(nodeId)
      node.attr({width: width})
    })
  }
}

export default SheetColumnHeader
