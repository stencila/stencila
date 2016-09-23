import Component from 'substance/ui/Component'
import Tool from 'substance/packages/tools/Tool'

import ViewTool from './tools/ViewTool'
import CopyTool from './tools/CopyTool'
import RefreshTool from './tools/RefreshTool'
import RevealTool from './tools/RevealTool'
// import CommentTool from './tools/CommentTool'
import EditTool from './tools/EditTool'
import SaveTool from './tools/SaveTool'
import CommitTool from './tools/CommitTool'
import ForkTool from './tools/ForkTool'
import SettingsTool from './tools/SettingsTool'

function SizerTool () {
  SizerTool.super.apply(this, arguments)
}

SizerTool.Prototype = function () {
  var _super = SizerTool.super.prototype

  this.getClassNames = function () {
    return _super.getClassNames.call(this) + ' se-sizer-tool'
  }

  this.renderIcon = function ($$) {
    return $$('i')
      .addClass(
        'fa fa-' + (this.props.maximized ? 'chevron-up' : 'circle')
      )
  }

  this.getTitle = function () {
    return (this.props.maximized ? 'Minimize' : 'Maximize')
  }

  this.onClick = function () {
    this.send('toggle-maximized')
  }
}

Tool.extend(SizerTool)

function DocumentToolset () {
  DocumentToolset.super.apply(this, arguments)

  this.handleActions({
    'toggle-maximized': this.toggleMaximized
  })
}

DocumentToolset.Prototype = function () {
  this.getInitialState = function () {
    return {
      maximized: true
    }
  }

  this.render = function ($$) {
    var el = $$('div')
      .addClass('sc-toolset sc-document-toolset')
      .addClass(this.state.maximized ? 'sm-maximized' : 'sm-minimized')
      .append(

        $$(SizerTool, {
          maximized: this.state.maximized
        }).ref('sizerTool'),

        $$(CopyTool, {
          copy: this.props.copy
        }).ref('copyTool'),

        $$(ViewTool, {
          view: this.props.view
        }).ref('viewTool'),

        $$(RefreshTool, this._getCommandState('refresh'))
          .ref('refreshTool'),

        $$(RevealTool, {
          name: 'reveal',
          active: this.props.reveal
        }).ref('revealTool'),

        // Currently not used
        /*
        $$(CommentTool, {
          name: 'comment',
          active: this.props.comment
        }).ref('commentTool'),
        */

        $$(EditTool, {
          name: 'edit',
          active: this.props.edit
        }).ref('editTool')
      )

    var editGroup = $$('div')
      .addClass('se-edit-group')
      .ref('editGroup')
      .append(
        $$(Tool, this._getCommandState('undo')),
        $$(Tool, this._getCommandState('redo')),
        $$(SaveTool, this._getCommandState('save')),
        $$(CommitTool, this._getCommandState('commit'))
      )
    if (this.props.edit) {
      editGroup.addClass('sm-enabled')
    }
    el.append(editGroup)

    el.append(
      $$(ForkTool, this._getCommandState('fork'))
        .ref('forkTool'),
      $$(SettingsTool, this._getCommandState('settings'))
        .ref('settingsTool')
    )

    return el
  }

  /**
   * Convieience method to deal with necessary hack
   * to add command name to state for Substance `Tools` to render
   * icons
   */
  this._getCommandState = function (name) {
    var state = this.context.commandManager.getCommandStates()[name]
    if (!state) throw new Error('Command {' + name + '} not found')
    state.name = name // A necessary hack at time of writing
    return state
  }

  /**
   * Toggle the `maximized` state
   */
  this.toggleMaximized = function () {
    this.extendState({
      maximized: !this.state.maximized
    })
  }
}

Component.extend(DocumentToolset)

export default DocumentToolset
