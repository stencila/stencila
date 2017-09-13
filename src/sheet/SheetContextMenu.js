import { ToolPanel } from 'substance'

export default class SheetContextMenu extends ToolPanel {

  render($$) {
    let el = $$('div').addClass('sc-sheet-context-menu')
    el.append(this.renderEntries($$))
    return el
  }

}