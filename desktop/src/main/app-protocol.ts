/*
Serving app using file:// protocol has secuirity implications
@see https://github.com/moloch--/reasonably-secure-electron#app-protocolts
*/

import { Protocol } from 'electron'
import * as fs from 'fs'
import * as path from 'path'

export const scheme = 'stencila'
const DIST_PATH = path.join(__dirname, '../renderer/')

const mimeTypes = {
  '.js': 'text/javascript',
  '.mjs': 'text/javascript',
  '.html': 'text/html',
  '.htm': 'text/html',
  '.json': 'application/json',
  '.css': 'text/css',
  '.svg': 'application/svg+xml',
  '.ico': 'image/vnd.microsoft.icon',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.map': 'text/plain',
}

function mime(filename: string): string {
  // @ts-ignore
  const type = mimeTypes[path.extname(`${filename || ''}`).toLowerCase()]
  return type ? type : null
}

type RequestHandler = Parameters<Protocol['registerBufferProtocol']>['1']

export const requestHandler: RequestHandler = (req, next) => {
  const reqUrl = new URL(req.url)
  let reqPath = path.resolve(reqUrl.pathname)

  if (reqPath === '/') {
    reqPath = '/main_window/index.html'
  }

  const reqFilename = path.basename(reqPath)
  const filePath = path.join(DIST_PATH, reqPath)

  fs.readFile(filePath, (err, data) => {
    const mimeType = mime(reqFilename)
    if (!err && mimeType !== null) {
      next({
        mimeType: mimeType,
        charset: 'utf-8',
        data: data,
      })
    } else {
      console.error(err)
    }
  })
}
