import { ToolPanel } from 'substance'

export default class SheetContextMenu extends ToolPanel {

  getEntryTypeComponents() {
    return {
      'tool-group': this.getComponent('menu-group'),
      'tool-dropdown': this.getComponent('menu-group')
    }
  }

  render($$) {
    let el = $$('div').addClass('sc-sheet-context-menu')
    el.append(this.renderEntries($$))
    return el
  }

}
