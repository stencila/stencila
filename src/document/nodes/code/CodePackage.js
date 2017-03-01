import {
  AnnotationComponent, AnnotationCommand, AnnotationTool, CodePackage
} from 'substance'
import CodeMarkdownComponent from './CodeMarkdownComponent'
import CodeMacro from './CodeMacro'

const { Code, CodeHTMLConverter, CodeXMLConverter } = CodePackage

export default {
  name: 'code',
  configure: function (config) {
    config.addNode(Code)
    config.addConverter('html', CodeHTMLConverter)
    config.addConverter('xml', CodeXMLConverter)
    config.addComponent('code', AnnotationComponent)
    config.addComponent('code-markdown', CodeMarkdownComponent)
    config.addCommand('code', AnnotationCommand, { nodeType: Code.type })
    config.addTool('code', AnnotationTool)
    config.addMacro(new CodeMacro())
    config.addIcon('code', { 'fontawesome': 'fa-code' })
    config.addLabel('code', {
      en: 'Code',
      de: 'Code'
    })
  }
}
