import ComponentClient from '../component/ComponentClient'

class SessionClient extends ComponentClient {

  execute (code, inputs) {
    return this.call('execute', {
      code: code || '',
      inputs: inputs || {}
    })
  }

}

export default SessionClient
