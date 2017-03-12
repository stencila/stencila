import Include from './Include'
import IncludeComponent from './IncludeComponent'
import IncludeHTMLConverter from './IncludeHTMLConverter'
import IncludeXMLConverter from './IncludeXMLConverter'

export default {
  name: 'include',
  configure: function (config) {
    config.addNode(Include)
    config.addComponent('include', IncludeComponent)
    config.addConverter('html', IncludeHTMLConverter)
    config.addConverter('xml', IncludeXMLConverter)
  }
}
