import InputSettingsBarComponent from './InputSettingsBarComponent'

export default {
  name: 'input-settings-bar',
  configure: function (config) {
    config.addComponent('input-settings-bar', InputSettingsBarComponent)
    config.addIcon('toggle-settings', { 'fontawesome': 'fa-cog' })
  }
}
