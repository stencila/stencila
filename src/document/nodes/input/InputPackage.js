import Input from './Input'
import InputComponent from './InputComponent'
import InputHTMLConverter from './InputHTMLConverter'
import InputXMLConverter from './InputXMLConverter'

export default {
  name: 'input',
  configure: function (config) {
    config.addNode(Input)
    config.addComponent('input', InputComponent)
    config.addConverter('html', InputHTMLConverter)
    config.addConverter('xml', InputXMLConverter)
    config.addTextType({
      name: 'input',
      data: {type: 'input'}
    })
    config.addLabel('input', {
      en: 'Input'
    })
  }
}
