const addon = require('../index.node')

/**
 * Serve a path
 * 
 * If the server is not yet started then it will be started and the path
 * added to its list of paths from which it will serve files.
 *
 * @returns A complete URL including a JSON Web Token authorizing access to
 *          all paths within the `path`.
 */
export function serve(path: string): string {
  return addon.serverServe(path)
}
