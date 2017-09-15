import { Component, getRelativeBoundingRect } from 'substance'

/*
  Highlights sheet cells with issues.
*/
export default class SheetIssuesOverlay extends Component {

  didMount() {
    this.props.linter.on('issues:changed', this.rerender, this)
  }

  dispose() {
    this.props.linter.off(this)
  }

  render($$) {
    let el = $$('div').addClass('sc-sheet-issues-overlay')
    // we need a rendered sheet view to be able to position cell overlays
    const sheetComponent = this.props.sheetComponent
    const sheetView = sheetComponent.getSheetView()
    if (sheetView) {
      let issues = this._getIssues()
      issues.forEach((issue) => {
        if (issue.isCellIssue()) {
          el.append(this._renderCellOverlay($$, issue))
        }
      })
    }
    return el
  }

  _renderCellOverlay($$, issue) {
    const sheetComponent = this.props.sheetComponent
    const sheetView = sheetComponent.getSheetView()
    let cell = issue.cell
    let td = sheetView.getCellComponentForCell(cell)
    if (td) {
      // TODO: I think we need the sheetComponent here instead of the sheetView
      let rect = getRelativeBoundingRect(td.el, sheetComponent.el)
      return $$(CellIssueOverlay, { issue })
        .css({
          top: rect.top,
          left: rect.left,
          height: rect.height,
          width: rect.width
        })
    }
  }

  _getIssues() {
    return this.props.linter.getIssues()
  }
}

class CellIssueOverlay extends Component {
  render($$) {
    const issue = this.props.issue
    let el = $$('div').addClass('sc-cell-issue-overlay')
    let severity = issue.isError() ? 'error' : 'warning'
    el.addClass(`sm-${severity}`)
    // TODO: maybe we want to add a marker
    // or a tooltip here
    return el
  }
}