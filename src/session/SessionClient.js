import ComponentDelegate from '../component/ComponentDelegate'

class SessionClient extends ComponentDelegate {

  execute (code, inputs) {
    return this.call('execute', {
      code: code || '',
      inputs: inputs || {}
    })
  }

}

export default SessionClient
