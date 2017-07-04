import { Configurator } from 'substance'
import DatatableDocument from './DatatableDocument'

export default class DatatableConfigurator extends Configurator {

  constructor () {
    super()

    // Define the schema (used by `getSchema()` to generate a `DocumentSchema` based on this
    // and the nodes added below by imports)
    this.defineSchema({
      DocumentClass: DatatableDocument,
      name: 'stencila-datatable',
      defaultTextType: 'text'
    })
  }

}