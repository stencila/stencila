import { EditInlineNodeCommand, Tool } from 'substance'
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
    config.addCommand('edit-inline-cell', EditInlineNodeCommand, { nodeType: 'inline-cell' })
    config.addTool('edit-inline-cell', EditInlineCellTool, { toolGroup: 'overlay' })

    config.addCommand('insert-inline-cell', InsertInlineCellCommand, {
      nodeType: 'inline-cell',
      disableCollapsedCursor: true
    })
    config.addTool('insert-inline-cell', Tool, { toolGroup: 'overlay' })
    config.addIcon('insert-inline-cell', { 'fontawesome': 'fa-caret-square-o-right' })
    config.addLabel('insert-inline-cell', 'Insert Output')
  }
}
