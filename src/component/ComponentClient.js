class ComponentClient {

  constructor (url) {
    this._url = url
  }

  request (method, url, data) {
    return new Promise((resolve, reject) => {
      var request = new window.XMLHttpRequest()
      request.open(method, url, true)
      request.setRequestHeader('Accept', 'application/json')

      request.onload = function () {
        if (request.status >= 200 && request.status < 400) {
          resolve(JSON.parse(request.responseText))
        } else {
          reject(request.status)
        }
      }

      if (data) {
        request.setRequestHeader('Content-Type', 'application/json')
        request.send(JSON.stringify(data))
      } else {
        request.send()
      }
    })
  }

  show () {
    return this.request('GET', this._url)
  }

  call (name, args) {
    return this.request('POST', this._url + '!' + name, args)
  }

}

export default ComponentClient
