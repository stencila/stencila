
class SessionClient {

  constructor (url) {
    this._url = url
  }

  execute (code) {
    return this._request('POST', this._url + '!execute', {
      code: code
    })
  }

  _request (method, url, data) {
    return new Promise((resolve, reject) => {
      var request = new window.XMLHttpRequest()
      request.open(method, url, true)
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

export default SessionClient
