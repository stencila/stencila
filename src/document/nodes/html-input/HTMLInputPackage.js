import { EditInlineNodeCommand, Tool } from 'substance'
import HTMLInput from './HTMLInput'
import HTMLInputComponent from './HTMLInputComponent'
import HTMLInputConverter from './HTMLInputConverter'
import EditHTMLInputTool from './EditHTMLInputTool'
import InsertHTMLInputCommand from './InsertHTMLInputCommand'

/*
  NOTE: We need to call this package `html-input` instead of `input` because
  there is already a Substance input component.
*/
export default {
  name: 'html-input',
  configure: function (config) {
    config.addNode(HTMLInput)
    config.addComponent('html-input', HTMLInputComponent)
    config.addConverter('html', HTMLInputConverter)
    config.addCommand('edit-html-input', EditInlineNodeCommand, { nodeType: 'html-input' })
    config.addTool('edit-html-input', EditHTMLInputTool, { toolGroup: 'overlay' })
    config.addIcon('html-input-settings', { 'fontawesome': 'fa-cog' })
    config.addLabel('html-input-settings', 'Settings')
    config.addCommand('insert-html-input', InsertHTMLInputCommand, {
      nodeType: 'html-input',
      disableCollapsedCursor: true
    })
    config.addTool('insert-html-input', Tool, { toolGroup: 'overlay' })
    config.addIcon('insert-html-input', { 'fontawesome': 'fa-sliders' })
    config.addLabel('insert-html-input', 'Insert Number Input')
  }
}
