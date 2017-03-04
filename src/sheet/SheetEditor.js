import {
  ScrollPane, AbstractEditor
} from 'substance'

import SheetComponent from './ui/SheetComponent'

/*
  Standalone SheetEditor.
*/
export default
class SheetEditor extends AbstractEditor {

  constructor(...args) {
    super(...args)

    // TODO: on the long run we may want to add tabs
    // for having multiple sheets
    // For now, we support only one sheet
    const doc = this.getDocument()
    let sheets = doc.getIndex('type').get('sheet')
    let sheetIds = Object.keys(sheets)
    this.activeSheet = null
    if (sheetIds.length>0) {
      this.activeSheet = sheets[sheetIds[0]]
    }
  }

  render($$) {
    let el = $$('div').addClass('sc-sheet-editor').append(
      this.renderMainSection($$)
    )
    return el
  }

  renderMainSection($$) {
    let sheetEditor
    if (this.activeSheet) {
      let node = this.activeSheet
      sheetEditor = $$(SheetComponent, {
        node: node
      }).ref(node.id)
    }
    let mainSection = $$('div').ref('main').addClass('se-main-section').append(
      $$(ScrollPane, {
        scrollbarPosition: 'left'
      }).append(sheetEditor)
    )
    return mainSection
  }
}
