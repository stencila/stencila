import { Component } from 'substance'

export default class SheetRowHeader extends NodeComponent {

export default class SheetRowHeader extends Component {
  render($$) {
    const rowIdx = this.props.rowIdx
    let th = $$('th')
      .attr('data-col', rowIdx)
      .addClass('sc-column-header')
      .text(String(rowIdx + 1))
    return th
  }
}