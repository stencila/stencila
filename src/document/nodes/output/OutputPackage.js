import Output from './Output'
import OutputComponent from './OutputComponent'
import OutputHTMLConverter from './OutputHTMLConverter'
import OutputXMLConverter from './OutputXMLConverter'

export default {
  name: 'output',
  configure: function (config) {
    config.addNode(Output)
    config.addComponent('output', OutputComponent)
    config.addConverter('html', OutputHTMLConverter)
    config.addConverter('xml', OutputXMLConverter)
    config.addTextType({
      name: 'output',
      data: {type: 'output'}
    })
    config.addLabel('output', {
      en: 'Output'
    })
  }
}
