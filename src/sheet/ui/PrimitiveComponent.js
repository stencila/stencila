import { Component } from 'substance'

/**
  Used for displaying cells which are not constant (i.e. `kind` != 'lit') and
  have a `type` that is a primitive (e.g. number, integer, string).
 */
export default
class PrimitiveComponent extends Component {

  render($$) {
    const cell = this.props.node
    const el = $$('div').addClass('sc-primitive')
    const prefix = cell.getPrefix()
    let value = cell.value
    if (value === undefined) {
      // TODO: use getLabel()
      value = 'Loading'
      el.addClass('sm-loading')
    }
    el.append(
      $$('span').addClass('se-name').text(prefix),
      $$('span').addClass('se-value').text(value)
    )
    return el
  }

}
