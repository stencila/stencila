import Document from 'substance/model/Document'

import SessionConfigurator from './SessionConfigurator'

class Session extends Document {

  constructor () {
    let configurator = new SessionConfigurator()
    let schema = configurator.getSchema()

    super(schema)

    this.create({
      id: 'executions',
      type: 'container',
      nodes: []
    })

    this.create({
      id: 'objects',
      type: 'container',
      nodes: []
    })
  }

}

export default Session
