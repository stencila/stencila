import SheetNode from './model/SheetNode'
import SheetCell from './model/SheetCell'
import SheetHTMLConverter from './model/SheetHTMLConverter'
import SheetCellHTMLConverter from './model/SheetCellHTMLConverter'
import SheetComponent from './ui/SheetComponent'
import SheetCellComponent from './ui/SheetCellComponent'

export default {
  name: 'sheet',
  configure: function(config) {
    config.addNode(SheetNode)
    config.addNode(SheetCell)
    config.addComponent('sheet', SheetComponent)
    config.addComponent('sheet-cell', SheetCellComponent)
    config.addConverter('html', SheetHTMLConverter)
    config.addConverter('html', SheetCellHTMLConverter)
  }
}