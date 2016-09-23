import Link from 'substance/packages/link/Link'
import LinkComponent from './LinkComponent'
import LinkCommand from './LinkCommand'
import LinkTool from './LinkTool'
import LinkHTMLConverter from 'substance/packages/link/LinkHTMLConverter'
import LinkXMLConverter from 'substance/packages/link/LinkXMLConverter'
import LinkMacro from './LinkMacro'

export default {
  name: 'link',
  configure: function (config) {
    config.addNode(Link)
    config.addComponent('link', LinkComponent)
    config.addConverter('html', LinkHTMLConverter)
    config.addConverter('xml', LinkXMLConverter)
    config.addCommand('link', LinkCommand, {nodeType: 'link'})
    config.addTool('link', LinkTool)
    config.addMacro(new LinkMacro())
    config.addIcon('link', { 'fontawesome': 'fa-link' })
    config.addLabel('link', {
      en: 'Link',
      de: 'Link'
    })
  }
}
