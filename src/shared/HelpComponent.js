import { Component, FontAwesomeIcon } from 'substance'
import FunctionHelpComponent from './FunctionHelpComponent'

export default class HelpComponent extends Component {

  render($$) {
    const page = this.props.page
    const [section, name] = page.split('/')

    let el = $$('div').addClass('sc-help').append(
      $$('div').addClass('se-context-header').append(
        $$('div').addClass('se-title').append('Help'),
        $$('div').addClass('se-icon').append(
          $$(FontAwesomeIcon, { icon: 'fa-close' })
        ).on('click', this._closeContext)
      )
    )

    // Do a little routing
    if (section === 'function') {
      el.append(
        $$(FunctionHelpComponent, {
          functionName: name
        })
      )
    } else {
      el.append('No page found for ', page)
    }
    return el
  }

  _closeContext() {
    this.send('toggleHelp')
  }
}
