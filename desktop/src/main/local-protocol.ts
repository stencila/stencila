/**
 * Provides secure access to local media files by rewriting `file://`
 * URLs to use this custom protocols which requires a HMAC
 * (hash-based message authentication code) for each requested path.
 */

import { createHmac, randomBytes } from 'crypto'
import { Protocol } from 'electron'

export const scheme = 'local'

const secret = randomBytes(256)

/**
 * Generate a HMAC for a path
 */
const generateHmac = (path: string) =>
  createHmac('sha256', secret).update(path).digest('hex')

/**
 * Rewrite a HTML page replacing `file://` URLs with `local://` URLs
 * with a HMAC appended.
 */
export const rewriteHtml = (html: string): string => {
  return html.replace(
    /(src=")file:\/\/(.*?)"/g,
    (_match, prefix: string, path: string) =>
      `${prefix}${scheme}://${path}?${generateHmac(path)}"`
  )
}

type RequestHandler = Parameters<Protocol['registerFileProtocol']>['1']

/**
 * Handle a request to this protocol by checking that the HMAC is correct
 * for the path and returning the path if it is and a 403 Forbidden otherwise.
 */
export const requestHandler: RequestHandler = (request, callback) => {
  const { pathname, search } = new URL(request.url)
  if (search.slice(1) == generateHmac(pathname))
    callback({ statusCode: 200, path: pathname })
  else callback({ statusCode: 403 })
}
