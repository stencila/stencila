import { Component, FontAwesomeIcon } from 'substance'
import ProjectTabs from './ProjectTabs'

export default class ProjectBar extends Component {

  render($$) {
    let contextId = this.props.contextId
    let helpToggle = $$('button')
      .addClass('se-toggle')
      .append('Help').on('click', this._toggleHelp)
    let hostsToggle = $$('button')
      .addClass('se-toggle se-hosts-toggle')
      .append(
        $$(FontAwesomeIcon, {icon: 'fa-server'})
      ).on('click', this._toggleHosts)

    if (contextId === 'help') {
      helpToggle.addClass('sm-active')
    } else if (contextId === 'hosts') {
      hostsToggle.addClass('sm-active')
    }

    let el = $$('div').addClass('sc-project-bar')
    el.append(
      $$(ProjectTabs, {
        documentArchive: this.props.documentArchive,
        documentId: this.props.documentId
      }),
      helpToggle,
      hostsToggle
    )
    // TODO: Render toggles for issues and help panel
    return el
  }

  _toggleHelp() {
    this.send('toggleHelp')
  }

  _toggleHosts() {
    this.send('toggleHosts')
  }

}
