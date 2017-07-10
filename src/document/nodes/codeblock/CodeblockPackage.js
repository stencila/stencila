import Codeblock from './Codeblock'
import CodeblockComponent from './CodeblockComponent'
import CodeblockHTMLConverter from './CodeblockHTMLConverter'
import InsertCodeblockCommand from './InsertCodeblockCommand'

export default {
  name: 'codeblock',
  configure: function (config) {
    config.addNode(Codeblock)
    config.addComponent('codeblock', CodeblockComponent)
    config.addConverter('html', CodeblockHTMLConverter)
    config.addCommand('codeblock', InsertCodeblockCommand, {
      commandGroup: 'insert',
      nodeType: 'codeblock'
    })
    config.addIcon('codeblock', { 'fontawesome': 'fa-quote-right' })
    config.addLabel('codeblock', {
      en: 'Codeblock',
      de: 'Codeblock'
    })
    config.addKeyboardShortcut('CommandOrControl+Alt+P', { command: 'codeblock' })
  }
}
