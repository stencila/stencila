import { EditInlineNodeCommand } from 'substance'
import InlineCell from './InlineCell'
import InlineCellComponent from './InlineCellComponent'
import InlineCellHTMLConverter from './InlineCellHTMLConverter'
import EditInlineCellTool from './EditInlineCellTool'
import InsertInlineCellCommand from './InsertInlineCellCommand'

export default {
  name: 'inline-cell',
  configure: function (config) {
    config.addNode(InlineCell)
    config.addComponent('inline-cell', InlineCellComponent)
    config.addConverter('html', InlineCellHTMLConverter)
    config.addCommand('edit-inline-cell', EditInlineNodeCommand, {
      nodeType: 'inline-cell' ,
      commandGroup: 'prompt'
    })
    config.addTool('edit-inline-cell', EditInlineCellTool)

    config.addCommand('insert-inline-cell', InsertInlineCellCommand, {
      nodeType: 'inline-cell',
      commandGroup: 'insert'
    })
    config.addIcon('insert-inline-cell', { 'fontawesome': 'fa-caret-square-o-right' })
    config.addLabel('insert-inline-cell', 'Output')
  }
}
