import Select from './Select'
import SelectComponent from './SelectComponent'
import SelectHTMLConverter from './SelectHTMLConverter'
import SelectXMLConverter from './SelectXMLConverter'

export default {
  name: 'select',
  configure: function (config) {
    config.addNode(Select)
    config.addComponent('select', SelectComponent)
    config.addConverter('html', SelectHTMLConverter)
    config.addConverter('xml', SelectXMLConverter)
    config.addTextType({
      name: 'select',
      data: {type: 'select'}
    })
    config.addLabel('select', {
      en: 'Select'
    })
  }
}
