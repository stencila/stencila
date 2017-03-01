import {
  AnnotationComponent, AnnotationCommand, AnnotationTool,
  StrongPackage
} from 'substance'

import StrongMacro from './StrongMacro'

const { Strong, StrongHTMLConverter, StrongXMLConverter } = StrongPackage


export default {
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
