import Math from './Math'
import MathHTMLConverter from './MathHTMLConverter'
import MathXMLConverter from './MathXMLConverter'
import MathComponent from './MathComponent'
import MathMarkdownComponent from './MathMarkdownComponent'
import MathCommand from './MathCommand'
import MathMacro from './MathMacro'
import MathTool from './MathTool'

export default {
  name: 'math',
  configure: function (config) {
    config.addNode(Math)
    config.addConverter('html', MathHTMLConverter)
    config.addConverter('xml', MathXMLConverter)
    config.addComponent('math', MathComponent)
    config.addComponent('math-markdown', MathMarkdownComponent)
    config.addCommand('math', MathCommand)
    config.addMacro(new MathMacro())
    config.addTool('math', MathTool)
    // TODO
    // Choose/create a better math icon (this is a random temporary)
    config.addIcon('math', { 'fontawesome': 'fa-tree' })
    config.addLabel('math', {
      en: 'Math'
    })
  }
}
