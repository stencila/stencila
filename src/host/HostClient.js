import ComponentClient from '../component/ComponentClient'
import SessionClient from '../session/SessionClient'

class HostClient extends ComponentClient {

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

}

export default HostClient
