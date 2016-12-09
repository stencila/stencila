import ComponentClient from '../component/ComponentClient'

class SessionClient extends ComponentClient {

  execute (code, args) {
    return this.call('execute', {
      code: code || '',
      args: args || {}
    })
  }

}

export default SessionClient
