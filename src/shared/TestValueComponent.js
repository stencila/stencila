import { Component } from 'substance'

export default
class TestValueComponent extends Component {
  render($$) {
    let value = this.props.value
    let el = $$('div').addClass('sc-test-value')
    let result = value.passed ? 'test-passed' : 'test-failed'
    el.addClass(value.passed ? 'sm-' + result : 'sm-' + result)
    el.append(
      $$('div').addClass('se-icon').append(
        this.context.iconProvider.renderIcon($$, result)
      ),
      $$('div').addClass('se-message').text(value.message)
    )
    return el
  }
}
