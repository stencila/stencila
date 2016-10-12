import Summary from './Summary'
import SummaryComponent from './SummaryComponent'
import SummaryHTMLConverter from './SummaryHTMLConverter'
import SummaryXMLConverter from './SummaryXMLConverter'

export default {
  name: 'summary',
  configure: function (config) {
    config.addNode(Summary)
    config.addComponent('summary', SummaryComponent)
    config.addConverter('html', SummaryHTMLConverter)
    config.addConverter('xml', SummaryXMLConverter)
    config.addTextType({
      name: 'summary',
      data: {type: 'summary'}
    })
    config.addIcon('summary', { 'fontawesome': 'fa-circle-o' })
    config.addLabel('summary', {
      en: 'Summary',
      de: 'Summary'
    })
  }
}
