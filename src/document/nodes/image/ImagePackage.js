import Image from './Image'
import ImageComponent from './ImageComponent'
import ImageMarkdownComponent from './ImageMarkdownComponent'
import ImageHTMLConverter from './ImageHTMLConverter'
import ImageXMLConverter from './ImageXMLConverter'
import ImageMacro from './ImageMacro'
import ImageTool from './ImageTool'

export default {
  name: 'image',
  configure: function (config) {
    config.addNode(Image)
    config.addComponent('image', ImageComponent)
    config.addComponent('image-markdown', ImageMarkdownComponent)
    config.addConverter('html', ImageHTMLConverter)
    config.addConverter('xml', ImageXMLConverter)
    config.addMacro(new ImageMacro())
    config.addTool('image', ImageTool)
    config.addIcon('image', { 'fontawesome': 'fa-image' })
    config.addLabel('image', {
      en: 'Image',
      de: 'Überschrift'
    })
  }
}
