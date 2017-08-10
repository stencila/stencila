import { Component, EditorSession } from 'substance'
import SpreadsheetComponent from './SpreadsheetComponent'

export default class SpreadsheetPage extends Component {

  getInitialState() {
    this._editorSession = new EditorSession(this.props.sheet, {
      configurator: this.props.configurator,
      context: {}
    })
    return {}
  }

  getChildContext() {
    return {
      editorSession: this._editorSession
    }
  }

  render($$) {
    const sheet = this.props.sheet
    let el = $$('div').addClass('sc-spreadsheet-page')
    el.append(
      $$(SpreadsheetComponent, { sheet }).css({
        width: this.props.width,
        height: this.props.height
      }).ref('spreadsheet')
    )
    return el
  }

}