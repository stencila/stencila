import {
  AnnotationCommand, AnnotationComponent, AnnotationTool,
  SuperscriptPackage
} from 'substance'

const { Superscript, SuperscriptHTMLConverter, SuperscriptXMLConverter } = SuperscriptPackage

export default {
  name: 'superscript',
  configure: function (config) {
    config.addNode(Superscript)
    config.addConverter('html', SuperscriptHTMLConverter)
    config.addConverter('xml', SuperscriptXMLConverter)
    config.addComponent('superscript', AnnotationComponent)
    config.addCommand('superscript', AnnotationCommand, { nodeType: 'superscript' })
    config.addTool('superscript', AnnotationTool)
    config.addIcon('superscript', { 'fontawesome': 'fa-superscript' })
    config.addLabel('superscript', {
      en: 'Superscript',
      de: 'Hochgestellt'
    })
  },
  Superscript: Superscript,
  SuperscriptHTMLConverter: SuperscriptHTMLConverter
}
