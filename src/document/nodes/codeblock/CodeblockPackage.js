import Codeblock from './Codeblock'
import CodeblockComponent from './CodeblockComponent'
import CodeblockHTMLConverter from './CodeblockHTMLConverter'

export default {
  name: 'codeblock',
  configure: function (config) {
    config.addNode(Codeblock)
    config.addComponent('codeblock', CodeblockComponent)
    config.addConverter('html', CodeblockHTMLConverter)
  }
}
