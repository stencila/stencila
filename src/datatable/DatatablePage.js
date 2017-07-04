import { Component } from 'substance'
import DatatableComponent from './DatatableComponent'

export default class DatatablePage extends Component {

  render($$) {
    // TODO: we need a consistent app structure
    const document = this.props.editorSession.getDocument()

    let el = $$('div').addClass('sc-datatable-page')

    el.append($$(DatatableComponent, { document }).ref('document'))

    return el
  }
}