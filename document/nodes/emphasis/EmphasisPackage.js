'use strict'

import Emphasis from 'substance/packages/emphasis/Emphasis'
import EmphasisHTMLConverter from 'substance/packages/emphasis/EmphasisHTMLConverter'
import EmphasisXMLConverter from 'substance/packages/emphasis/EmphasisXMLConverter'
import AnnotationComponent from 'substance/ui/AnnotationComponent'
import AnnotationCommand from 'substance/ui/AnnotationCommand'
import AnnotationTool from 'substance/ui/AnnotationTool'
import EmphasisMacro from './EmphasisMacro'

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
