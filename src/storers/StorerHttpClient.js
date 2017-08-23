import {PUT} from '../util/requests'

/**
 * A HTTP client for a remote `Storer`
 *
 * Implements the `Storer` API by remote procedure calls (RPC) to a remote
 * Storer (e.g. a `FileStorer` running in a different process)
 */
export default class StorerHttpClient {

  constructor(url) {
    this._url = url
  }

  getDirectory() {
    return PUT(this._url + '!getDirectory')
  }

  getMain() {
    return PUT(this._url + '!getMain')
  }

  getFiles() {
    return PUT(this._url + '!getFiles')
  }

  getInfo() {
    return PUT(this._url + '!getInfo').then(info => {
      // Store info here on the client, mainly
      // for debugging purposes
      this._info = info
      return info
    })
  }

  readFile(path) {
    return PUT(this._url + '!readFile', {path: path})
  }

  writeFile(path, content) {
    return PUT(this._url + '!writeFile', {path: path, content: content})
  }

  deleteFile(path) {
    return PUT(this._url + '!readFile', {path: path})
  }

}
