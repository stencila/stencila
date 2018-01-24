import { Component } from 'substance'
import ProjectTabs from './ProjectTabs'

export default class ProjectBar extends Component {

  render($$) {
    let contextId = this.props.contextId
    let helpToggle = $$('button')
      .addClass('se-toggle')
      .append('Help').on('click', this._toggleHelp)

    if (contextId === 'help') {
      helpToggle.addClass('sm-active')
    }

    let el = $$('div').addClass('sc-project-bar')
    el.append(
      $$(ProjectTabs, {
        documentArchive: this.props.documentArchive,
        documentId: this.props.documentId
      }),
      helpToggle
    )
    // TODO: Render toggles for issues and help panel
    return el
  }

  _toggleHelp() {
    this.send('toggleHelp')
  }

}
