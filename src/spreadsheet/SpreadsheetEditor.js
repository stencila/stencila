import { AbstractEditor, Toolbar } from 'substance'
import SpreadsheetComponent from './SpreadsheetComponent'

export default class SpreadsheetEditor extends AbstractEditor {

  render($$) {
    const configurator = this.getConfigurator()
    const sheet = this.getDocument()
    let el = $$('div').addClass('sc-spreadsheet-editor')
    el.append(
      $$(Toolbar, {
        toolPanel: configurator.getToolPanel('toolbar')
      }).ref('toolbar'),
      $$(SpreadsheetComponent, { sheet }).css({
        // We decided to pass down dimension, so that
        // the viewport can be computed easily.
        // Then we need some way of double-rendering so that we can retrieve
        // the actual available size
        // TODO: rethink this -- isn't it possible to compute this dynamically from within
        // the SpreadsheetComponent?
        // width: this.getWidth(),
        // height: this.getHeight()
      }).ref('spreadsheet'),
      $$('div').addClass('se-status-bar').text('STATUS-BAR')
    )
    return el
  }

  getWidth() {
    if (this.el) {
      return this.el.getWidth()
    } else {
      return 0
    }
  }

  getHeight() {
    if (this.el) {
      return this.el.getHeight()
    } else {
      return 0
    }
  }

}