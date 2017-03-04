import SheetNode from './model/SheetNode'
import Cell from './model/Cell'
import SheetHTMLConverter from './model/SheetHTMLConverter'
import CellHTMLConverter from './model/CellHTMLConverter'
import SheetComponent from './ui/SheetComponent'
import CellComponent from './ui/CellComponent'

export default {
  name: 'sheet',
  configure: function(config) {
    config.addNode(SheetNode)
    config.addNode(Cell)
    config.addComponent('sheet', SheetComponent)
    config.addComponent('cell', CellComponent)
    config.addConverter('html', SheetHTMLConverter)
    config.addConverter('html', CellHTMLConverter)
  }
}