import { SwitchTextTypeCommand } from 'substance'
import Title from './Title'
import TitleComponent from './TitleComponent'
import TitleHTMLConverter from './TitleHTMLConverter'
import TitleXMLConverter from './TitleXMLConverter'

export default {
  name: 'title',
  configure: function (config) {
    config.addNode(Title)
    config.addComponent('title', TitleComponent)
    config.addConverter('html', TitleHTMLConverter)
    config.addConverter('xml', TitleXMLConverter)
    config.addCommand('title', SwitchTextTypeCommand, {
      spec: { type: 'title' },
      commandGroup: 'text-types'
    })
    config.addIcon('title', { 'fontawesome': 'fa-asterisk' })
    config.addLabel('title', 'Title')
  }
}
