import { Component } from 'substance'

export default
class TestValueComponent extends Component {
  render($$) {
    let value = this.props.value
    let el = $$('div').addClass('sc-test-value')
    el.addClass(value.passed ? 'sm-test-passed' : 'sm-test-failed')
    el.text(value.message)
    return el
  }
}
