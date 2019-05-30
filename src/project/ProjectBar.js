import { Component } from 'substance'
import ProjectTab from './ProjectTab'
import AddProjectTab from './AddProjectTab'
import ContextToggle from './ContextToggle'
import documentTypes from '../documentTypes'

export default class ProjectBar extends Component {

  render($$) {
    const archive = this.props.archive
    const documentEntries = archive.getDocumentEntries()
    let contextId = this.props.contextId

    let el = $$('div').addClass('sc-project-bar')
    let projectTabs = $$('div').addClass('se-project-tabs')

    documentEntries.forEach(entry => {
      if (_isVisible(entry)) {
        projectTabs.append(
          $$(ProjectTab, {
            entry,
            active: this.props.documentId === entry.id
          })
        )
      }
    })

    projectTabs.append(
      $$(AddProjectTab)
    )

    let contextToggles = $$('div').addClass('se-context-toggles')

    contextToggles.append(
      $$(ContextToggle, {
        action: 'toggleHelp',
        icon: 'fa-question-circle',
        active: contextId === 'help'
      }),
      $$(ContextToggle, {
        action: 'toggleHosts',
        icon: 'fa-server',
        active: contextId === 'hosts'
      })
    )

    el.append(
      projectTabs,
      contextToggles
    )

    return el
  }

}

function _isVisible(entry) {
  return Boolean(documentTypes[entry.type])
}
