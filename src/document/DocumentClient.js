
class DocumentClient {

  constructor (url) {
    this._url = url
  }

  save (content, format) {
    this._request('POST', this._url + '!save', {
      content: content,
      format: format || 'html'
    })
  }

  _request (method, url, data, cb) {
    var request = new window.XMLHttpRequest()
    request.open(method, url, true)
    request.setRequestHeader('Content-Type', 'application/json')

    request.onload = function () {
      if (request.status >= 200 && request.status < 400) {
        if (cb) cb(null, JSON.parse(request.responseText))
      } else {
        let error = new Error('Request failed. Returned status: ' + request.status)
        if (cb) return cb(error)
        else throw error
      }
    }

    if (data) {
      request.send(JSON.stringify(data))
    } else {
      request.send()
    }
  }
}

export default DocumentClient
