'use strict'

import Mark from './Mark'
import MarkHTMLConverter from './MarkHTMLConverter'
import MarkXMLConverter from './MarkXMLConverter'
import MarkCommand from './MarkCommand'
import MarkComponent from './MarkComponent'
import MarkTool from './MarkTool'

export default {
  name: 'mark',
  configure: function (config) {
    config.addNode(Mark)
    config.addConverter('html', MarkHTMLConverter)
    config.addConverter('xml', MarkXMLConverter)
    config.addComponent('mark', MarkComponent)
    config.addCommand('mark', MarkCommand)
    config.addTool('mark', MarkTool)
    config.addIcon('mark', { 'fontawesome': 'fa-comment-o' })
    config.addLabel('mark', {
      en: 'Comment'
    })
  }
}
