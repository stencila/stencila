let cache = {}

/**
 * A `require` function for the browser
 *
 * We call it `need` to avoid confusion but, when in the browser, it is included in the scope of `JsSession#execute`
 * as `require`.
 *
 * As an alternative approach see https://github.com/maxogden/browser-module-sandbox (used by http://requirebin.com/ and others)
 * It involves putting each module's source, as well as your own source, into a `<script>` within an `<iframe>`.
 * This seems to have a couple of issues for our use case (a) each module gets parsed every call to `JsSession#execute`
 * (b) needs some message passing to get results of evaluation back from the `<iframe>`. This current approach
 * is far simpler but this type of alternative might need to be looked at in the future.
 *
 * @param  {String} name    Module name
 * @param  {String} version Version needed, defaults to "latest"
 * @return {Object}         The module
 */
function need (name, version) {
  version = version || 'latest'

  let id = `${name}@${version}`
  if (cache[id]) {
    return cache[id]
  } else {
    let request = new XMLHttpRequest()
    request.open('GET', `https://wzrd.in/bundle/${id}/`, false)
    request.send(null)
    if (request.status === 200) {
      eval(request.responseText) // eslint-disable-line no-eval
      let module = require(name) // This `require` comes from the wzrd bundle
      cache[id] = module
      return module
    } else {
      throw new Error(request.responseText)
    }
  }
}

module.exports = need
