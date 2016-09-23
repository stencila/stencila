'use strict'

import extend from 'lodash/extend'
import he from 'he'
var Raven = require('raven-js')

import location from './utilities/location'

/**
 * Stencila browser application entry point
 */
export default function browser (App) {
  window.onload = function () {
    let props = {}
    // Is this a local host?
    const hostname = window.location.hostname
    props.local = hostname === 'localhost' || hostname === '127.0.0.1'
    // Get `address` and `copy` from the path
    var path = window.location.pathname
    var matches = path.match(/([^@]+)(@(\w+))?/)
    props.address = matches[1]
    props.copy = matches[3]
    // Update with URL query parameters
    var params = location.params()
    extend(props, params)
    // Check if `?static=1`
    if (params.static !== '1') {
      if (!App) return

      // Get component as HTML content or a JSON snapshot data
      // for rerendering by the `App` and then hide content element (if any)
      var content = document.getElementById('content')
      if (content) {
        props.format = 'html'
        props.data = content.innerHTML
        content.style.display = 'none'
      } else {
        var dataElem = document.getElementById('data')
        if (dataElem) {
          props.format = 'json'
          props.data = JSON.parse(he.decode(dataElem.textContent || dataElem.innerHTML))
        } else {
          throw Error('Neither #content or #data is available to initialize the component')
        }
      }

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
