import Configurator from 'substance/util/Configurator'

class DatatableConfigurator extends Configurator {

  constructor () {
    super()

    this.defineSchema({
      name: 'stencila-datatable',
      defaultTextType: 'cell'
    })
  }

}

export default DatatableConfigurator
