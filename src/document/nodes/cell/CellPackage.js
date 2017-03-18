import { Tool } from 'substance'
import InsertCellCommand from './InsertCellCommand'
import Cell from './Cell'
import CellComponent from './CellComponent'
import CellHTMLConverter from './CellHTMLConverter'
import TableValueComponent from './TableValueComponent'
import CodeHighlightComponent from './CodeHighlightComponent'

export default {
  name: 'cell',
  configure: function (config) {
    config.addNode(Cell)
    config.addComponent('cell', CellComponent)
    config.addComponent('value:table', TableValueComponent)
    config.addComponent('code-highlight', CodeHighlightComponent)
    config.addConverter('html', CellHTMLConverter)
    config.addCommand('insert-cell', InsertCellCommand)
    config.addTool('insert-cell', Tool, { toolGroup: 'insert' })
    config.addIcon('insert-cell', { 'fontawesome': 'fa-caret-square-o-right' })
    config.addIcon('error', { 'fontawesome': 'fa-exclamation' })
    config.addLabel('insert-cell', 'Insert cell')
  }
}
