'use strict';

module.exports = {
  name: 'visualEditor',
  configure: function(config) {

    // Commit
    config.addCommand('commit', require('./CommitCommand'));
    config.addTool('commit', require('./CommitTool'));
    config.addLabel('commit', {
      'en': 'Commit'
    });
    config.addIcon('commit', { 'fontawesome': 'fa-dot-circle-o'});

    // Edit
    config.addTool('edit', require('./EditTool'));
    config.addLabel('edit', {
      'en': 'Edit'
    });
    config.addIcon('edit', { 'fontawesome': 'fa-pencil'});

    // Fork
    config.addCommand('fork', require('./ForkCommand'));
    config.addTool('fork', require('./ForkTool'));
    config.addLabel('fork', {
      'en': 'Fork'
    });
    config.addIcon('fork', { 'fontawesome': 'fa-code-fork'});

    // Reveal
    config.addTool('reveal', require('./RevealTool'));
    config.addLabel('reveal', {
      'en': 'Reveal'
    });
    config.addIcon('reveal', { 'fontawesome': 'fa-eye'});

    // Refresh
    config.addCommand('refresh', require('./RefreshCommand'));
    config.addTool('refresh', require('./RefreshTool'));
    config.addLabel('refresh', {
      'en': 'Refresh'
    });
    config.addIcon('refresh', { 'fontawesome': 'fa-refresh'});

    // Save
    config.addCommand('save', require('./SaveCommand'));
    config.addTool('save', require('./SaveTool'));
    config.addLabel('save', {
      'en': 'Save'
    });
    config.addIcon('save', { 'fontawesome': 'fa-save'});

    // Settings
    config.addCommand('settings', require('./SettingsCommand'));
    config.addTool('settings', require('./SettingsTool'));
    config.addLabel('settings', {
      'en': 'Settings'
    });
    config.addIcon('settings', { 'fontawesome': 'fa-cog'});

  }
}