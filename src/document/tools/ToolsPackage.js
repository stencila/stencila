import { BasePackage } from 'substance'

import CommitCommand from './CommitCommand'
import CommitTool from './CommitTool'
import EditTool from './EditTool'
import ForkCommand from './ForkCommand'
import ForkTool from './ForkTool'
import CommentTool from './CommentTool'
import RevealTool from './RevealTool'
import RefreshCommand from './RefreshCommand'
import RefreshTool from './RefreshTool'
import SaveCommand from './SaveCommand'
import SaveTool from './SaveTool'
import SettingsCommand from './SettingsCommand'
import SettingsTool from './SettingsTool'

const { UndoCommand, RedoCommand } = BasePackage

export default {
  name: 'tools',
  configure: function (config) {
    // Commit
    config.addCommand('commit', CommitCommand)
    config.addTool('commit', CommitTool)
    config.addLabel('commit', {
      'en': 'Commit'
    })
    config.addIcon('commit', { 'fontawesome': 'fa-dot-circle-o' })

    // Edit
    config.addTool('edit', EditTool)
    config.addLabel('edit', {
      'en': 'Edit'
    })
    config.addIcon('edit', { 'fontawesome': 'fa-pencil' })

    // Undo
    config.addCommand('undo', UndoCommand)
    config.addIcon('undo', { 'fontawesome': 'fa-undo' })

    // Redo
    config.addCommand('redo', RedoCommand)
    config.addIcon('redo', { 'fontawesome': 'fa-repeat' })

    // Fork
    config.addCommand('fork', ForkCommand)
    config.addTool('fork', ForkTool)
    config.addLabel('fork', {
      'en': 'Fork'
    })
    config.addIcon('fork', { 'fontawesome': 'fa-code-fork' })

    // Comment
    config.addTool('comment', CommentTool)
    config.addLabel('comment', {
      'en': 'Comment'
    })
    config.addIcon('comment', { 'fontawesome': 'fa-comment-o' })

    // Reveal
    config.addTool('reveal', RevealTool)
    config.addLabel('reveal', {
      'en': 'Reveal'
    })
    config.addIcon('reveal', { 'fontawesome': 'fa-eye' })

    // Refresh
    config.addCommand('refresh', RefreshCommand)
    config.addTool('refresh', RefreshTool)
    config.addLabel('refresh', {
      'en': 'Refresh'
    })
    config.addIcon('refresh', { 'fontawesome': 'fa-refresh' })

    // Save
    config.addCommand('save', SaveCommand)
    config.addTool('save', SaveTool)
    config.addLabel('save', {
      'en': 'Save'
    })
    config.addIcon('save', { 'fontawesome': 'fa-save' })

    // Settings
    config.addCommand('settings', SettingsCommand)
    config.addTool('settings', SettingsTool)
    config.addLabel('settings', {
      'en': 'Settings'
    })
    config.addIcon('settings', { 'fontawesome': 'fa-cog' })
  }
}
