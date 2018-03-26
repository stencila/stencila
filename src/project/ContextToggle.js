import { Component, FontAwesomeIcon } from 'substance'

export default class ContextToggle extends Component {

  render($$) {
    // NOTE: We use sc-project-tab here to inherit its styles
    let el = $$('div').addClass('sc-context-toggle sc-project-tab')
    el.append(
      $$(FontAwesomeIcon, {icon: this.props.icon })
    )
    .on('click', this._toggleAction)
    if (this.props.active) {
      el.addClass('sm-active')
    }
    return el
  }

  _toggleAction() {
    this.send(this.props.action)
  }

}
