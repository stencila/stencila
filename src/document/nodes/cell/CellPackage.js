import { Tool } from 'substance'
import InsertCellCommand from './InsertCellCommand'

export default {
  name: 'cell',
  configure: function (config) {
    config.addCommand('insert-cell', InsertCellCommand)
    config.addTool('insert-cell', Tool, { toolGroup: 'insert' })
    config.addIcon('insert-cell', { 'fontawesome': 'fa-caret-square-o-right' })
    config.addLabel('insert-cell', 'Insert cell')
  }
}
