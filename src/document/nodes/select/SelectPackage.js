import { EditInlineNodeCommand } from 'substance'
import Select from './Select'
import SelectComponent from './SelectComponent'
import SelectHTMLConverter from './SelectHTMLConverter'
import EditSelectTool from './EditSelectTool'
// import InsertSelectCommand from './InsertSelectCommand'

export default {
  name: 'select',
  configure: function (config) {
    config.addNode(Select)
    config.addComponent('select', SelectComponent)
    config.addConverter('html', SelectHTMLConverter)
    config.addCommand('edit-select', EditInlineNodeCommand, { nodeType: 'select' })
    config.addTool('edit-select', EditSelectTool, { toolGroup: 'overlay' })

    // Disabled until we have good UX for it
    // config.addCommand('insert-select', InsertSelectCommand, {
    //   nodeType: 'select',
    //   disableCollapsedCursor: true
    // })
    // config.addTool('insert-select', Tool, { toolGroup: 'overlay' })
    // config.addIcon('insert-select', { 'fontawesome': 'fa-caret-down' })
    // config.addLabel('insert-select', 'Insert Select')
  }
}
