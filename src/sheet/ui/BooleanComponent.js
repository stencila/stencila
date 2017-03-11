import { Component } from 'substance'

/**
 * Displays cells which have a boolean (true/false)
 * value type
 */
export default
class BooleanComponent extends Component {

  render($$) {
    var cell = this.props.node
    var el = $$('div').addClass('sc-boolean')

    var prefix = cell.getPrefix()
    el.append(
      $$('span').addClass('se-prefix').text(prefix)
    )

    // Using lowercase below allows for alternative string representations
    // in different languages eg. TRUE in R, True in Python, true in Javascript
    const value = cell.value
    let icon
    let className
    if (value === undefined) {
      icon = 'spinner'
      className = 'sm-loading'
    }
    else if (value.toLowerCase() === 'true') {
      icon = 'check'
      className = 'sm-true'
    }
    else if (value.toLowerCase() === 'false') {
      icon = 'times'
      className = 'sm-false'
    }
    el.addClass(className)
    el.append(
      this.context.iconProvider.renderIcon($$, icon)
    )

    return el
  }

}
