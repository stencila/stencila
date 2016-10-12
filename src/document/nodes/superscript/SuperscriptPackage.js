import Superscript from 'substance/packages/superscript/Superscript'
import SuperscriptHTMLConverter from 'substance/packages/superscript/SuperscriptHTMLConverter'
import SuperscriptXMLConverter from 'substance/packages/superscript/SuperscriptXMLConverter'
import AnnotationCommand from 'substance/ui/AnnotationCommand'
import AnnotationComponent from 'substance/ui/AnnotationComponent'
import AnnotationTool from 'substance/ui/AnnotationTool'

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
