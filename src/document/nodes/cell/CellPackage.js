import { Tool } from 'substance'
import InsertCellCommand from './InsertCellCommand'
import Cell from './Cell'
import CellComponent from './CellComponent'
import CellHTMLConverter from './CellHTMLConverter'
import TableValueComponent from './TableValueComponent'
import IntegerValueComponent from './IntegerValueComponent'
import FloatValueComponent from './FloatValueComponent'
import StringValueComponent from './StringValueComponent'
import ArrayValueComponent from './ArrayValueComponent'
import CodeHighlightComponent from './CodeHighlightComponent'

export default {
  name: 'cell',
  configure: function (config) {
    config.addNode(Cell)
    config.addComponent('cell', CellComponent)
    config.addComponent('value:table', TableValueComponent)
    config.addComponent('value:integer', IntegerValueComponent)
    config.addComponent('value:float', FloatValueComponent)
    config.addComponent('value:string', StringValueComponent)
    config.addComponent('value:array', ArrayValueComponent)
    config.addComponent('code-highlight', CodeHighlightComponent)
    config.addConverter('html', CellHTMLConverter)
    config.addCommand('insert-cell', InsertCellCommand)
    config.addTool('insert-cell', Tool, { toolGroup: 'insert' })
    config.addIcon('insert-cell', { 'fontawesome': 'fa-caret-square-o-right' })
    config.addLabel('insert-cell', 'Insert Cell')

    // TODO: some of these could go into a shared package
    config.addIcon('error', { 'fontawesome': 'fa-exclamation' })
    config.addIcon('ellipsis', { 'fontawesome': 'fa-ellipsis-v' })
  }
}
