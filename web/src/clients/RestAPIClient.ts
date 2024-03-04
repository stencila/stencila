import { APIAccess } from '../api/fetchMethods'
import { DocumentId } from '../types'
import { Secret } from '../types/api'

/**
 * This class is a wrapper for all internal API requests.
 *
 * @export
 * @class RestAPIClient
 */
export class RestAPIClient {
  /**
   * Open the document at the supplied _path_
   *
   * @static
   * @param {string} path
   * @return {*}
   * @memberof RestAPIClient
   */
  static async openDocument(path: string) {
    return APIAccess.requestApiResponseAsJson<{ id: DocumentId }>({
      url: `/~documents/open/${path}`,
      identifier: path,
    })
  }

  /**
   * close a document on the server
   *
   * @param {DocumentId} docId
   * @memberof RestAPIClient
   */
  static async closeDocument(docId: DocumentId) {
    await APIAccess.requestApi({
      url: `/~documents/${docId}/close`,
      identifier: docId,
      init: { method: 'POST' },
    })
  }

  /**
   * Retrieve a list of configured secrets for the application.
   *
   * @static
   * @return {*}
   * @memberof RestAPIClient
   */
  static async listSecrets() {
    return APIAccess.requestApiResponseAsJson<Secret[]>({ url: '/~secrets' })
  }

  /**
   * Sets the value of a secret in the API.
   *
   * @static
   * @param {string} name
   * @param {string} value
   * @return {*}
   * @memberof RestAPIClient
   */
  static async setSecret(name: string, value: string) {
    return APIAccess.requestApi({
      url: `/~secrets/${name}`,
      identifier: name,
      init: {
        method: 'POST',
        headers: { 'Content-Type': 'text/plain', body: value },
      },
    })
  }

  /**
   * Delete the secret defined by _name_
   *
   * @static
   * @param {string} name
   * @return {*}
   * @memberof RestAPIClient
   */
  static async deleteSecret(name: string) {
    return APIAccess.requestApi({
      url: `/~secrets/${name}`,
      identifier: name,
      init: {
        method: 'DELETE',
      },
    })
  }
}
