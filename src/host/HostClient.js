import ComponentClient from '../component/ComponentClient'
import SessionClient from '../session/SessionClient'

class HostClient extends ComponentClient {

  call (name, args) {
    return this.request('POST', this._url + '/!' + name, args)
  }

  open (address) {
    return this.request('GET', this._url + '/' + address)
  }

  new (type) {
    return new Promise((resolve, reject) => {
      this.request('GET', this._url + '/new://' + type)
        .then((data) => {
          resolve(new SessionClient(data.url))
        })
        .catch((error) => {
          reject(error)
        })
    })
  }

  discover () {
    return this.call('discover')
  }

}

export default HostClient
