import { Component } from 'substance'

export default class CellOverflowComponent extends Component {
  render($$) {
    return $$('div').addClass('sc-cell-overflow')
      .append(this.props.children)
  }
}
