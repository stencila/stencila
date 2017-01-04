import ImageNode from 'substance/packages/image/ImageNode'
import ImageComponent from 'substance/packages/image/ImageComponent'
import ImageMarkdownComponent from './ImageMarkdownComponent'
import ImageHTMLConverter from 'substance/packages/image/ImageHTMLConverter'
import ImageXMLConverter from 'substance/packages/image/ImageXMLConverter'
import ImageMacro from './ImageMacro'
import ImageTool from './ImageTool'

export default {
  name: 'image',
  configure: function (config) {
    config.addNode(ImageNode)
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
