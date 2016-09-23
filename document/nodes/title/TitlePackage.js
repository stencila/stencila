'use strict'

import Title from './Title'
import TitleComponent from './TitleComponent'
import TitleHTMLConverter from './TitleHTMLConverter'
import TitleXMLConverter from './TitleXMLConverter'

module.exports = {
  name: 'title',
  configure: function (config) {
    config.addNode(Title)
    config.addComponent('title', TitleComponent)
    config.addConverter('html', TitleHTMLConverter)
    config.addConverter('xml', TitleXMLConverter)
    config.addTextType({
      name: 'title',
      data: {type: 'title'}
    })
    config.addIcon('title', { 'fontawesome': 'fa-asterisk' })
    config.addLabel('title', {
      en: 'Title',
      de: 'Title'
    })
  }
}
