import ComponentClient from '../component/ComponentClient'

class SessionClient extends ComponentClient {

  execute (code) {
    return this.call('execute', {
      code: code
    })
  }

}

export default SessionClient
