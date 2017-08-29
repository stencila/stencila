import { ToolPanel } from 'substance'

export default class SpreadsheetContextMenu extends ToolPanel {

  render($$) {
    let el = $$('div').addClass('sc-spreadsheet-context-menu')
    el.append(this.renderEntries($$))
    return el
  }

}