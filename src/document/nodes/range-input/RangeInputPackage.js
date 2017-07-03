import { EditInlineNodeCommand, Tool } from 'substance'
import RangeInput from './RangeInput'
import RangeInputComponent from './RangeInputComponent'
import RangeInputConverter from './RangeInputConverter'
import EditRangeInputTool from './EditRangeInputTool'
import InsertRangeInputCommand from './InsertRangeInputCommand'

export default {
  name: 'range-input',
  configure: function (config) {
    config.addNode(RangeInput)
    config.addComponent('range-input', RangeInputComponent)
    config.addConverter('html', RangeInputConverter)
    config.addCommand('edit-range-input', EditInlineNodeCommand, {
      nodeType: 'range-input',
      commandGroup: 'prompt'
    })
    config.addTool('edit-range-input', EditRangeInputTool)
    config.addCommand('insert-range-input', InsertRangeInputCommand, {
      nodeType: 'range-input',
      commandGroup: 'insert'
    })
    config.addTool('insert-range-input', Tool, { toolGroup: 'overlay' })
    config.addIcon('insert-range-input', { 'fontawesome': 'fa-sliders' })
    config.addLabel('insert-range-input', 'Range Input')
  }
}
