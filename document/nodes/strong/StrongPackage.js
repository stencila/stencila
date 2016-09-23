'use strict'

import Strong from 'substance/packages/strong/Strong'
import StrongHTMLConverter from 'substance/packages/strong/StrongHTMLConverter'
import StrongXMLConverter from 'substance/packages/strong/StrongXMLConverter'
import AnnotationComponent from 'substance/ui/AnnotationComponent'
import AnnotationCommand from 'substance/ui/AnnotationCommand'
import AnnotationTool from 'substance/ui/AnnotationTool'
import StrongMacro from './StrongMacro'

module.exports = {
  name: 'strong',
  configure: function (config) {
    config.addNode(Strong)
    config.addConverter('html', StrongHTMLConverter)
    config.addConverter('xml', StrongXMLConverter)
    config.addComponent('strong', AnnotationComponent)
    config.addCommand('strong', AnnotationCommand, { nodeType: 'strong' })
    config.addTool('strong', AnnotationTool)
    config.addMacro(new StrongMacro())
    config.addIcon('strong', { 'fontawesome': 'fa-bold' })
    config.addLabel('strong', {
      en: 'Strong',
      de: 'Starke'
    })
  }
}
