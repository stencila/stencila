import SessionClient from '../session/SessionClient'

class HostClient {

  constructor (url) {
    this._url = url
  }

  show (address) {
    return this._request('GET', this._url)
  }

  open (address) {
    return this._request('GET', this._url + '/' + address)
  }

  new (type) {
    return new Promise((resolve, reject) => {
      this._request('GET', this._url + '/new://' + type)
        .then((data) => {
          resolve(new SessionClient(data.url))
        })
        .catch((error) => {
          reject(error)
        })
    })
  }

  _request (method, url, data) {
    return new Promise((resolve, reject) => {
      var request = new window.XMLHttpRequest()
      request.open(method, url, true)
      request.setRequestHeader('Accept', 'application/json')
      request.setRequestHeader('Content-Type', 'application/json')

      request.onload = function () {
        if (request.status >= 200 && request.status < 400) {
          resolve(JSON.parse(request.responseText))
        } else {
          reject(request.status)
        }
      }

      if (data) {
        request.send(JSON.stringify(data))
      } else {
        request.send()
      }
    })
  }
}

export default HostClient
