import { Component } from 'substance'

export default
class NullValueComponent extends Component {
  render($$) {
    return $$('div').addClass('sc-null-value').text('null')
  }
}
