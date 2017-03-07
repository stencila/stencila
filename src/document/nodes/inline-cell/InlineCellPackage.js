import { EditInlineNodeCommand } from 'substance'
import InlineCell from './InlineCell'
import InlineCellComponent from './InlineCellComponent'
import InlineCellHTMLConverter from './InlineCellHTMLConverter'
import EditInlineCellTool from './EditInlineCellTool'

export default {
  name: 'inline-cell',
  configure: function (config) {
    config.addNode(InlineCell)
    config.addComponent('inline-cell', InlineCellComponent)
    config.addConverter('html', InlineCellHTMLConverter)
    config.addCommand('edit-inline-cell', EditInlineNodeCommand, { nodeType: 'inline-cell' })
    config.addTool('edit-inline-cell', EditInlineCellTool, { toolGroup: 'overlay' })
  }
}
