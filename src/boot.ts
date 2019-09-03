/**
 * Module for installing Encoda native modules and executables
 *
 * The [`pkg`](https://github.com/zeit/pkg) Node.js packager does not
 * package native modules.  i.e `*.node` files. There are various ways to handle this but
 * we found the easiest/safest was to simply copy the directories for the
 * packages with native modules, from the host system, into directory where the
 * binary is installed. This script does that via `encoda-deps.tar.gz` which is
 * packaged in the binary snapshot as an `asset`.
 *
 * See:
 *   - https://github.com/stencila/encoda/pull/47#issuecomment-489912132
 *   - https://github.com/zeit/pkg/issues/329
 *   - https://github.com/JoshuaWise/better-sqlite3/issues/173
 *   - `package.json`
 */
import fs from 'fs-extra'
import path from 'path'
import tar from 'tar'

import { getLogger } from '@stencila/logga'

const logger = getLogger('stencila')

/**
 * Is this process being run as a `pkg` packaged binary?
 */
const packaged =
  ((process.mainModule !== undefined &&
    process.mainModule.id.endsWith('.exe')) ||
    Object.prototype.hasOwnProperty.call(process, 'pkg')) &&
  fs.existsSync(path.join('/', 'snapshot'))

/**
 * The home directory for this modules or process where
 * native modules and executables are placed.
 */
const home = packaged ? path.dirname(process.execPath) : path.dirname(__dirname)

/**
 *  Unzip the native dependencies to home
 */
export function extractDeps(forceExtract: boolean = false): void {
  const shouldExtract =
    packaged &&
    (forceExtract || !fs.existsSync(path.join(home, 'node_modules')))
  if (shouldExtract) {
    tar.x({
      sync: true,
      file: path.join('/', 'snapshot', 'stencila', 'stencila-deps.tgz'),
      strip: 1,
      C: home
    })

    logger.info('Dependencies extracted.')
  }
}
