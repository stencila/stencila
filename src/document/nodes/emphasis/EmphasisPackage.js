import {
  AnnotationCommand, AnnotationComponent, AnnotationTool,
  EmphasisPackage
} from 'substance'

import EmphasisMacro from './EmphasisMacro'

const { EmphasisHTMLConverter, EmphasisXMLConverter } = EmphasisPackage

export default {
  name: 'emphasis',
  configure: function (config) {
    config.addNode(Emphasis)
    config.addConverter('html', EmphasisHTMLConverter)
    config.addConverter('xml', EmphasisXMLConverter)
    config.addComponent('emphasis', AnnotationComponent)
    config.addCommand('emphasis', AnnotationCommand, { nodeType: 'emphasis' })
    config.addTool('emphasis', AnnotationTool)
    config.addMacro(new EmphasisMacro())
    config.addIcon('emphasis', { 'fontawesome': 'fa-italic' })
    config.addLabel('emphasis', {
      en: 'Emphasis',
      de: 'Betonung'
    })
  }
}
