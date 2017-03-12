import ComponentDelegate from '../component/ComponentDelegate'
import SessionClient from '../session/SessionClient'

class HostClient extends ComponentDelegate {

  call (name, args) {
    return this.request('POST', this._url + '/!' + name, args)
  }

  open (address) {
    return new Promise((resolve, reject) => {
      this.request('GET', this._url + '/' + address)
        .then((data) => {
          let component
          if (data.type === 'document') {
            component = new ComponentDelegate(data.url)
          } else if (data.kind === 'session') {
            component = new SessionClient(data.url)
          }
          resolve(component)
        })
        .catch((error) => {
          reject(error)
        })
    })
  }

  new (type) {
    console.error('DEPRECIATED')
    return this.open('+' + type)
  }

  discover () {
    return this.call('discover')
  }

}

export default HostClient
