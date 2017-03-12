import { Component } from 'substance'

export default class Dashboard extends Component {

  getBackend() {
    return this.props.backend
  }

  render ($$) {
    var el = $$('div').addClass('sc-dashboard')
    el.append('HELLO I AM THE DASHBOARD')
    return el
  }

}
