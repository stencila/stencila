import {
  AnnotationCommand, AnnotationComponent, AnnotationTool,
  SubscriptPackage
} from 'substance'
const { Subscript, SubscriptHTMLConverter, SubscriptXMLConverter } = SubscriptPackage

export default {
  name: 'subscript',
  configure: function (config) {
    config.addNode(Subscript)
    config.addConverter('html', SubscriptHTMLConverter)
    config.addConverter('xml', SubscriptXMLConverter)
    config.addComponent('subscript', AnnotationComponent)
    config.addCommand('subscript', AnnotationCommand, { nodeType: 'subscript' })
    config.addTool('subscript', AnnotationTool)
    config.addIcon('subscript', { 'fontawesome': 'fa-subscript' })
    config.addLabel('subscript', {
      en: 'Subscript',
      de: 'Tiefgestellt'
    })
  },
  Subscript: Subscript,
  SubscriptHTMLConverter: SubscriptHTMLConverter
}
