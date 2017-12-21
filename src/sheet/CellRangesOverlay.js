import { Component } from 'substance'

/*
  Used to render one ore multiple cell ranges
  which would be positioned relative to a SheetComponent.
*/
export default class CellRangesOverlay extends Component {

  render($$) {
    let el = $$('div').addClass('sc-cell-ranges-overlay')
    // Note: this is already anticipating a scenario with multiple ranges
    // rendered at one time
    if (this.props.ranges) {
      this.props.ranges.forEach((rect) => {
        el.append(
          $$('div').addClass('se-range').css(rect)
        )
      })
    }
    return el
  }

}