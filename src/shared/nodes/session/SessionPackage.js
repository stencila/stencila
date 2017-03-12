import Session from './Session'
import SessionComponent from './SessionComponent'
import SessionHTMLConverter from './SessionHTMLConverter'
import SessionXMLConverter from './SessionXMLConverter'

export default {
  name: 'session',
  configure: function (config) {
    config.addNode(Session)
    config.addComponent('session', SessionComponent)
    config.addConverter('html', SessionHTMLConverter)
    config.addConverter('xml', SessionXMLConverter)
    config.addIcon('session', { 'fontawesome': 'fa-bolt' })
    config.addLabel('session', {
      en: 'Session'
    })
  }
}
