'use strict'

import Execute from './Execute'
import ExecuteComponent from './ExecuteComponent'
import ExecuteHTMLConverter from './ExecuteHTMLConverter'
import ExecuteXMLConverter from './ExecuteXMLConverter'

module.exports = {
  name: 'execute',
  configure: function (config) {
    config.addNode(Execute)
    config.addComponent('execute', ExecuteComponent)
    config.addConverter('html', ExecuteHTMLConverter)
    config.addConverter('xml', ExecuteXMLConverter)
    config.addTextType({
      name: 'execute',
      data: {type: 'execute'}
    })
    config.addIcon('execute', { 'fontawesome': 'fa-play' })
    config.addLabel('execute', {
      en: 'Execute'
    })
  }
}
