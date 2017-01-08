import extend from 'lodash/extend'
import he from 'he'
import Raven from 'raven-js'

import location from './utilities/location'

/**
 * Stencila browser application entry point
 */
export default function browser (App) {
  window.onload = function () {
    let props = {}
    props.host = window.location.host
    // Is this a local host? Under electron, hostname is empty
    const hostname = window.location.hostname
    props.local = hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '';
    // Try to get descriptors from the <head>
    ['id', 'address', 'url', 'static', 'edit'].forEach(function (item) {
      let elem = document.querySelector('meta[name=' + item + ']')
      if (elem) props[item] = elem.content
    })
    // Fallback to getting `url`, `address` and `version` from the path
    var path = window.location.pathname
    var matches = path.match(/\/([^@]+)(@(\w+))?/)
    if (matches) {
      if (!props.url) props.url = window.location.origin + '/' + matches[1]
      if (!props.address) props.address = matches[1]
      props.version = matches[3]
    } else {
      if (!props.url) props.url = window.location.origin
    }
    // Update with URL query parameters
    var params = location.params()
    extend(props, params)
    // Check if `static`
    if (props.static !== '1') {
      if (!App) return
      // Get component data from page for rerendering by the `App` and then hide it
      var data = document.getElementById('data')
      if (data) {
        props.format = data.getAttribute('data-format')
        if (props.format === 'html') {
          props.data = data.outerHTML
        } else {
          props.data = JSON.parse(he.decode(data.textContent || data.innerHTML))
        }
        data.style.display = 'none'
      }
      // If not local then capture any errors
      if (props.local) {
        window.app = App.mount(props, document.body)
      } else {
        Raven
          .config('https://6329017160394100b21be92165555d72@app.getsentry.com/37250')
          .install()
        try {
          window.app = App.mount(props, document.body)
        } catch (e) {
          Raven.captureException(e)
        }
      }
    }
  }
}
