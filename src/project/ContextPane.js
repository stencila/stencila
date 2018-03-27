import { Component, FontAwesomeIcon } from 'substance'
import HelpComponent from '../shared/HelpComponent'
import HostsComponent from '../host/HostsComponent'

const LABELS = {
  'help': 'Help',
  'hosts': 'Hosts'
}

export default class ContextPane extends Component {

  render($$) {
    let el = $$('div').addClass('sc-context-pane')
    if (this.props.contextId) {
      el.append(
        $$('div').addClass('se-header').append(
          LABELS[this.props.contextId],
          $$('button').addClass('se-icon').append(
            $$(FontAwesomeIcon, { icon: 'fa-times-circle' })
          ).on('click', this._closeContext)
        ),
        this._renderContextContent($$)
      )
    } else {
      el.addClass('sm-hidden')
    }
    return el
  }

  _renderContextContent($$) {
    let el = $$('div').addClass('se-content')
    if (this.props.contextId === 'help') {
      el.append(
        $$(HelpComponent, this.props.contextProps)
      )
    } else if (this.props.contextId === 'hosts') {
      el.append(
        $$(HostsComponent, this.props.contextProps)
      )
    } else {
      el.append(`Unknown context ${this.props.contextId}`)
    }
    return el
  }

  _closeContext() {
    this.send('closeContext')
  }



}
