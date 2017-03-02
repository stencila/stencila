import { Component } from 'substance'

/*
  Little pop-over that displays the available functions

  TODO: this is not interactive yet. we would need to capture
        key events (down, up) to navigate the suggestion list.
        Also clicking on an entry should fill them into the
        cell.
*/
export default
class SelectFunction extends Component {
  render($$) {
    var el = $$('div').addClass('sc-select-function')

    this.props.entries.forEach(function(entry) {
      el.append($$('div').addClass('se-entry').append(entry))
    })
    return el
  }
}
