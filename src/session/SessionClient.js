import ComponentClient from '../component/ComponentClient'

class SessionClient extends ComponentClient {

  execute (code, pipes) {
    return this.call('execute', {
      code: code || '',
      pipes: pipes || []
    })
  }

}

export default SessionClient
