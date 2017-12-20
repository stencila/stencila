import { Component, Toolbar} from 'substance'
import SheetIssuesCounter from './SheetIssuesCounter'

export default class SheetStatusBar extends Component {
  render($$) {
    const configurator = this.context.configurator
    let el = $$('div').addClass('sc-sheet-statusbar').append(
      $$(SheetIssuesCounter).ref('counter'),
      $$(Toolbar, {
        toolPanel: configurator.getToolPanel('statusbar')
      }).ref('statusbar')
    )
    return el
  }
}
