import { NodeComponent } from 'substance'

/*
  Renders a keyboard-selectable reproducable figure target item
*/
export default class ReproFigPreview extends NodeComponent {

  render($$) {
    let node = this.props.node
    let el = $$('div')
      .addClass('sc-repro-fig-preview')
      .attr({'data-id': node.id})

    if (this.props.selected) {
      el.addClass('sm-selected')
    }

    el.append(
      this._renderLabel($$)
    )
    return el
  }

  _renderLabel($$) {
    const node = this.props.node
    const label = node && node.state ? this.getLabel(node.state.label) : ''

    return $$('div').addClass('se-label').append(label)
  }
}
