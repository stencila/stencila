import {Component, Tool} from 'substance'

import SizerTool from './tools/SizerTool'
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

class DocumentToolset extends Component {

  constructor (...args) {
    super(...args)

    this.handleActions({
      'toggle-maximized': () => {
        this.extendState({
          maximized: !this.state.maximized
        })
      }
    })
  }

  getInitialState () {
    return {
      maximized: true
    }
  }

  render ($$) {
    let commandStates = this.context.commandManager.getCommandStates()
    function toolProps (name) {
      let toolProps = Object.assign({}, commandStates[name])
      toolProps.name = name
      toolProps.icon = name
      return toolProps
    }

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

        $$(RefreshTool, toolProps('refresh'))
          .ref('refreshTool'),

        $$(RevealTool, {
          name: 'reveal',
          icon: 'reveal',
          active: this.props.reveal
        }).ref('revealTool'),

        $$(EditTool, {
          name: 'edit',
          icon: 'edit',
          active: this.props.edit
        }).ref('editTool')
      )

    var editGroup = $$('div')
      .addClass('se-edit-group')
      .ref('editGroup')
      .append(
        $$(Tool, toolProps('undo')),
        $$(Tool, toolProps('redo')),
        $$(SaveTool, toolProps('save')),
        $$(CommitTool, toolProps('commit'))
      )
    if (this.props.edit) {
      editGroup.addClass('sm-enabled')
    }
    el.append(editGroup)

    el.append(
      $$(ForkTool, toolProps('fork'))
        .ref('forkTool'),
      $$(SettingsTool, toolProps('settings'))
        .ref('settingsTool')
    )

    return el
  }

}

export default DocumentToolset
