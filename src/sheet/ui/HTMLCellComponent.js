import { Component } from 'substance'

/**
 * Used for displaying cells which are have `html` as their
 * value `type`.
 */
export default
class HTMLCellComponent extends Component {

  render($$) {
    var cell = this.props.node
    var el = $$('div').addClass('sc-html-cell')

    var value = cell.value
    var className = ''
    if (value === undefined) {
      value = 'Loading'
      className = 'sm-loading'
    }
    el.addClass(className).html(value)

    if(window.MathJax && window.MathJax.Hub) {
      MathJax.Hub.Queue(["Rerender", MathJax.Hub, cell.id])
    }

    return el
  }

}
