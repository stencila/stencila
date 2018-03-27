/**
 * Make a HTTP request
 *
 * A simple wrapper around `XMLHttpRequest` with certain conventions for
 * our RPC API:
 *   - send and receive JSON
 *   - return a Promise
 *
 * @param  {string} method - Request method (a.k.a. verb)
 * @param  {string} url - Requested URL
 * @param  {object} data - Data to POST or PUT
 * @return {Promise}
 */
export function request (method, url, data) {
  var XMLHttpRequest
  if (typeof window === 'undefined') XMLHttpRequest = require("xmlhttprequest").XMLHttpRequest
  else XMLHttpRequest = window.XMLHttpRequest

  return new Promise((resolve, reject) => {
    var request = new XMLHttpRequest()
    request.open(method, url, true)
    request.setRequestHeader('Accept', 'application/json')
    // Send any credentials (e.g. cookies) in request headers
    // (necessary for remote peers)
    request.withCredentials = true

    request.onload = function () {
      let result
      try {
        result = JSON.parse(request.responseText)
      } catch (error) {
        result = request.responseText
      }
      if (request.status >= 200 && request.status < 400) {
        resolve(result)
      } else {
        reject({
          status: request.status,
          body: result
        })
      }
    }

    request.onerror = function () {
      reject(new Error('An error occurred with request "' + method + ' ' + url + '"'))
    }

    if (data) {
      request.setRequestHeader('Content-Type', 'application/json')
      request.send(JSON.stringify(data))
    } else {
      request.send()
    }
  })
}

export function GET (url) {
  return request('GET', url)
}

export function POST (url, data) {
  return request('POST', url, data)
}

export function PUT (url, data) {
  return request('PUT', url, data)
}

export function DELETE (url) {
  return request('DELETE', url)
}
