import Default from './Default'
import DefaultHTMLConverter from './DefaultHTMLConverter'
import DefaultXMLConverter from './DefaultXMLConverter'
import DefaultComponent from './DefaultComponent'
import DefaultMarkdownComponent from './DefaultMarkdownComponent'

export default {
  name: 'default',
  configure: function (config) {
    config.addNode(Default)
    config.addConverter('html', DefaultHTMLConverter)
    config.addConverter('xml', DefaultXMLConverter)
    config.addComponent('default', DefaultComponent)
    config.addComponent('default-markdown', DefaultMarkdownComponent)
    config.addIcon('default', { 'fontawesome': 'fa-circle-o' })
    config.addLabel('default', {
      en: 'Default node'
    })
  }
}
