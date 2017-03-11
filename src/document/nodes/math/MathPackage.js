import { Tool } from 'substance'
import Math from './Math'
import MathHTMLConverter from './MathHTMLConverter'
import MathComponent from './MathComponent'
import { EditInlineNodeCommand } from 'substance'
import EditMathTool from './EditMathTool'
import InsertMathCommand from './InsertMathCommand'
// import MathMacro from './MathMacro'

export default {
  name: 'math',
  configure: function (config) {
    config.addNode(Math)
    config.addConverter('html', MathHTMLConverter)
    config.addComponent('math', MathComponent)

    config.addCommand('edit-math', EditInlineNodeCommand, { nodeType: 'math' })
    config.addTool('edit-math', EditMathTool, { toolGroup: 'overlay' })

    config.addCommand('insert-math', InsertMathCommand, {
      nodeType: 'math',
      disableCollapsedCursor: true
    })
    config.addTool('insert-math', Tool, { toolGroup: 'overlay' })

    // config.addMacro(new MathMacro())
    // TODO: Choose/create a better math icon (this is a random temporary)
    config.addIcon('insert-math', { 'fontawesome': 'fa-dollar' })
    config.addLabel('insert-math', 'Math')
  }
}
