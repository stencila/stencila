import Configurator from 'substance/util/Configurator'

import ExecutionPackage from './nodes/execution/ExecutionPackage'

class DocumentConfigurator extends Configurator {

  constructor () {
    super()

    this.defineSchema({
      name: 'stencila-document',
      defaultTextType: 'paragraph'
    })

    this.import(ExecutionPackage)
  }

}

export default DocumentConfigurator
