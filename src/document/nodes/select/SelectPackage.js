import { EditInlineNodeCommand } from 'substance'
import Select from './Select'
import SelectComponent from './SelectComponent'
import SelectHTMLConverter from './SelectHTMLConverter'
import EditSelectTool from './EditSelectTool'

export default {
  name: 'select',
  configure: function (config) {
    config.addNode(Select)
    config.addComponent('select', SelectComponent)
    config.addConverter('html', SelectHTMLConverter)
    config.addCommand('edit-select', EditInlineNodeCommand, { nodeType: 'select' })
    config.addTool('edit-select', EditSelectTool, { toolGroup: 'overlay' })
  }
}
