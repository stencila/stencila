import { Component } from 'substance'

/**
 * Used for displaying cells which are have `html` as their
 * value `type`.
 */
export default
class HTMLCellComponent extends Component {

  render($$) {
    const cell = this.props.node
    const el = $$('div').addClass('sc-html-cell')
    if (value === undefined) {
      el.addClass('sm-loading')
    }
    el.html(value)
    return el
  }

}
