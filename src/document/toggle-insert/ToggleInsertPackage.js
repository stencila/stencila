import ToggleInsertCommand from './ToggleInsertCommand'
import ToggleInsertTool from './ToggleInsertTool'

export default {
  name: 'insert-node',
  configure: function(config) {
    config.addCommand('toggle-insert', ToggleInsertCommand)
    config.addTool('toggle-insert', ToggleInsertTool, {
      toolGroup: 'overlay'
    })
    config.addIcon('toggle-insert', { 'fontawesome': 'fa-plus' })
    config.addLabel('toggle-insert', 'Insert Content')
  }
}
