import Print from './Print'
import PrintHTMLConverter from './PrintHTMLConverter'
import PrintXMLConverter from './PrintXMLConverter'
import PrintComponent from './PrintComponent'
import PrintCommand from './PrintCommand'
import PrintMacro from './PrintMacro'
import PrintTool from './PrintTool'

export default {
  name: 'print',
  configure: function (config) {
    config.addNode(Print)
    config.addConverter('html', PrintHTMLConverter)
    config.addConverter('xml', PrintXMLConverter)
    config.addComponent('print', PrintComponent)
    config.addCommand('print', PrintCommand)
    config.addMacro(new PrintMacro())
    config.addTool('print', PrintTool)
    config.addIcon('print', { 'fontawesome': 'fa-eyedropper' })
    config.addLabel('print', {
      en: ''
    })
  }
}
