import { Component } from 'substance'

/**
 * Used for displaying cells which are not constant (i.e. `kind` != 'lit') and
 * have a `type` that is a primitive (e.g. number, integer, string).
 */
export default
class PrimitiveExpressionComponent() {

  render($$) {
    var cell = this.props.node

    var el = $$('div').addClass('sc-primitive')

    var prefix = cell.getPrefix()
    el.append(
      $$('span').addClass('se-name').text(prefix)
    )

    var value = cell.value
    var className = 'se-value'
    if (value === undefined) {
      value = 'Loading'
      className = 'se-loading'
    }
    el.append(
      $$('span').addClass(className).text(value)
    )

    return el
  }

}
