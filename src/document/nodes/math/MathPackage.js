import { Tool } from 'substance'
import Math from './Math'
import MathHTMLConverter from './MathHTMLConverter'
import MathComponent from './MathComponent'
import MathCommand from './MathCommand'
// import MathMacro from './MathMacro'


export default {
  name: 'math',
  configure: function (config) {
    config.addNode(Math)
    config.addConverter('html', MathHTMLConverter)
    config.addComponent('math', MathComponent)
    config.addCommand('math', MathCommand)
    // config.addMacro(new MathMacro())
    config.addTool('math', Tool)
    // TODO: Choose/create a better math icon (this is a random temporary)
    config.addIcon('math', { 'fontawesome': 'fa-tree' })
    config.addLabel('math', {
      en: 'Math'
    })
  }
}
