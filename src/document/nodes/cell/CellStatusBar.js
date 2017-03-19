import {Component} from 'substance'

export default
class CellStatusBar extends Component {

  render($$) {
    let el = $$('div').addClass('sc-cell-status-bar')
    el.append('status')
    return el
  }

}