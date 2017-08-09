import { Component } from 'substance'
import SpreadsheetComponent from './SpreadsheetComponent'

export default class SpreadsheetPage extends Component {

  render($$) {
    const sheet = this.props.sheet
    let el = $$('div').addClass('sc-spreadsheet-page')
    el.append(
      $$(SpreadsheetComponent, { sheet })
    )
    return el
  }

}