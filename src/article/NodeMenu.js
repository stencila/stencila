import { ToolPanel } from 'substance'

/*
  Used for actions on isolated nodes
*/
export default class NodeMenu extends ToolPanel {

  getEntryTypeComponents() {
    return {
      'tool-group': this.getComponent('menu-group'),
      'tool-dropdown': this.getComponent('menu-group')
    }
  }

  render($$) {
    let el = $$('div').addClass('sc-node-menu')
    el.append(this.renderEntries($$))
    return el
  }

}
