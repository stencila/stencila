import { Component } from 'substance'
import ProjectTabs from './ProjectTabs'

export default class ProjectBar extends Component {

  render($$) {
    let el = $$('div').addClass('sc-project-bar')
    el.append(
      $$(ProjectTabs, {
        documentContainer: this.props.documentContainer,
        documentId: this.props.documentId
      })
    )
    // TODO: Render toggles for issues and help panel
    return el
  }

}
