import { EventEmitter, uuid } from 'substance'
import { KJUR } from 'jsrsasign'

import FunctionManager from '../function/FunctionManager'
import Engine from '../engine/Engine'
import JavascriptContextClient from '../contexts/JavascriptContextClient'
import MiniContext from '../contexts/MiniContext'
import ContextHttpClient from '../contexts/ContextHttpClient'
import libcore from 'stencila-libcore'

/**
 * Each Stencila process has a single instance of the `Host` class which
 * orchestrates instances of executions contexts, including those in
 * in other processses.
 */
export default class Host extends EventEmitter {

  constructor (options = {}) {
    super()

    /**
     * The id of this host. Used by other Stencila
     * hosts to uniquely identify this host.
     * e.g for sequence numbers when authenticating requests
     */
    this._id = 'client-host-' + uuid()

    /**
     * Options used to configure this host
     *
     * @type {object}
     */
    this._options = options

    /**
     * Stencila hosts known to this host.
     * A `Map` of `url` to `key`
     *
     * @type {Map}
     */
    this._hosts = new Map()

    /**
     * Execution environments provided by other hosts.
     * A `Map` of `url` to `[environ]`
     *
     * @type {Map}
     */
    this._environs = new Map()


    this._environ = 'local'

    /**
     * Stencila hosts that are peers (i.e. within the same
     * execution environment). A `Map` of `url` to `manifest`.
     *
     * @type {Map}
     */
    this._peers = new Map()

    /**
     * Instances managed by this host
     *
     * @type {object}
     */
    this._instances = {}

    /**
     * Execution contexts are currently managed separately to
     * ensure that there is only one for each language
     *
     * @type {object}
     */
    this._contexts = {}

    /**
     * Counts of instances of each class.
     * Used for consecutive naming of instances
     *
     * @type {object}
     */
    this._counts = {}

    /**
     * Execution engine for scheduling execution across contexts
     *
     * @type {Engine}
     */
    this._engine = options.engine || new Engine({ host: this })

    /**
     * Manages functions imported from libraries
     *
     * @type {FunctionManager}
     */
    this._functionManager = new FunctionManager()

  }

  // For compatability with Stencila Host Manifest API (as is stored in this._peers)

  /**
   * The URL of this internal host
   */
  get url() {
    return 'internal'
  }

  /**
   * The resource types supported by this internal host
   */
  get types() {
    return {
      JavascriptContext: { name: 'JavascriptContext' },
      MiniContext: { name: 'MiniContext' }
    }
  }

  // Getters...

  /**
   * Get this host's configuration options
   */
  get options () {
    return this._options
  }

  /**
   * Get known hosts
   */
  get hosts () {
    return this._hosts
  }

  /**
   * Get the environments registered with this host
   */
  get environs () {
    return this._environs
  }

  /**
   * Get this host's peers
   */
  get peers () {
    return this._peers
  }

  /**
   * Get the resource instances (e.g. contexts, storers) managed by this host
   */
  get instances() {
    return this._instances
  }

  /**
   * Get the execution contexts managed by this host
   */
  get contexts() {
    return this._contexts
  }

  /**
   * Get this host's execution engine
   */
  get engine () {
    return this._engine
  }

  /**
   * Get this host's function manager
   */
  get functionManager() {
    return this._functionManager
  }

  /**
   * Initialize this host
   *
   * @return {Promise} Initialisation promise
   */
  initialize () {
    const options = this._options

    let promises = [
      // Always create a Javascript execution context for
      // execution of core functions
      this.createContext('js')
    ]

    // Seed with specified hosts
    let hosts = options.hosts
    if (hosts) {
      for (let url of hosts) {
        let key = null
        if (url === 'origin') url = options.origin
        else if (url.indexOf('|') > -1) {
          let parts = url.split('|')
          url = parts[0]
          key = parts[1]
        }
        let promise = this.registerHost(url, key)
        promises.push(promise)
      }
    }

    // Start discovery of other peers
    if (options.discover) {
      this.discoverHosts(options.discover)
    }

    return Promise.all(promises).then(() => {
      const urls = Array.from(this._hosts.keys())
      if (urls.length > 0) {
        const url = urls[0]
        const host = this._hosts.get(url)
        const environs = host.manifest.environs
        if (environs.length > 0) {
          this.selectEnviron(environs[0].id)
          return this.selectHost(url)
        }
      }
    }).then(() => {
      // Run the engine after connecting to any peer hosts so that they are connected
      // (and have registered functions) before the engine attempts
      // to create contexts for external languages like R, SQL etc
      this._engine.run(10) // Refresh interval of 10ms
    })
  }

  selectEnviron (environId) {
    if (environId !== this._environ) {
      this._environ = environId
      this.emit('environ:changed')
    }
  }

  registerHost (url, key = null, optimistic = false) {
    return this._request('GET', url + '/manifest').then(manifest => {
      if (manifest.stencila) {
        const host = {
          key,
          manifest,
          sent: 0,
          messages: []
        }
        this._hosts.set(url, host)
        this.emit('hosts:changed')
      }
    }).catch((error) => {
      if (!optimistic) throw error
    })
  }

  deregisterHost (url) {
    this._hosts.delete(url)
    this.emit('hosts:changed')
  }

  selectHost (url) {
    this._hosts.get(url).selected = true
    this.emit('hosts:changed')
    return this._post(url, '/environ/' + this._environ).then(location => {
      let peerUrl
      if (location.url) peerUrl = location.url
      else if (location.path) peerUrl = url + location.path
      else peerUrl = url
      return this._get(peerUrl, '/manifest').then(manifest => {
        this._peers.set(peerUrl, manifest)
        this.emit('peers:changed')
      })
    })
  }

  deselectHost (url) {
    if (this._hosts.has(url)) {
      this._hosts.get(url).selected = false
      this.emit('hosts:changed')
      return this._delete(url, '/environ/' + this._environ).then(() => {
        this._peers.delete(url)
        this.emit('peers:changed')
      })
    }
  }

  /**
   * Discover peers
   *
   * Currently, this method just does a port scan on the localhost to find
   * peers. More sophisticated peer discovery mechanisms for remote peers
   * will be implemented later.
   *
   * Unfortunately if a port is not open then you'll get a console error like
   * `GET http://127.0.0.1:2040/ net::ERR_CONNECTION_REFUSED`. In Chrome, this can
   * not be avoided programatically (see http://stackoverflow.com/a/43056626/4625911).
   * The easiest approach is silence these errors in Chrome is to check the
   * 'Hide network' checkbox in the console filter.
   *
   * Set the `interval` parameter to a value greater than zero to trigger ongoing discovery and
   * to a negative number to turn off discovery.
   *
   * @param {number} interval - The interval (seconds) between discovery attempts
   */
  discoverHosts (interval=10) {
    this.options.discover = interval
    if (interval >= 0) {
      for (let port=2000; port<=2100; port+=10) {
        this.registerHost(`http://127.0.0.1:${port}`, null, true)
      }
      if (interval > 0) {
        this.discoverHosts(-1) // Ensure any existing interval is turned off
        this._discoverHostsInterval = setInterval(() => this.discoverHosts(0), interval*1000)
      }
    } else {
      if (this._discoverHostsInterval) {
        clearInterval(this._discoverHostsInterval)
        this._discoverHostsInterval = null
      }
    }
  }

  /**
   * Create a new instance of a resource
   *
   * @param  {string} type - Name of class of instance
   * @param  {string} name - Name for new instance
   * @return {Promise} Resolves to an instance
   */
  create (type, args) {
    // Register a created instance
    let _register = (id, host, type, instance) => {
      this._instances[id] = {host, type, instance}
      this.emit('instance:created')
    }

    // Look for type in peer hosts
    for (let [url, manifest] of this._peers) {
      for (let spec of Object.values(manifest.types)) {
        if (spec.name === type) {
          return this._post(url, '/' + type, args).then(id => {
            let Client
            if (spec.client === 'ContextHttpClient') Client = ContextHttpClient
            else throw new Error(`Unsupported type: ${spec.client}`)

            let instance = new Client(this, url, id)
            _register(id, url, type, instance)
            return {id, instance}
          })
        }
      }
    }

    // Fallback to providing an in-browser instances of resources where available
    let instance
    if (type === 'JavascriptContext') {
      instance = new JavascriptContextClient(this)
      instance.importLibrary(libcore)
    } else if (type === 'MiniContext') {
      // MiniContext requires a pointer to this host so that
      // it can obtain other contexts for executing functions
      instance = new MiniContext(this)
    } else {
      // Resolve an error so that this does not get caught in debugger during
      // development and instead handle error elsewhere
      return Promise.resolve(new Error(`No peers able to provide: ${type}`))
    }

    // Generate an id for the instance
    let number = (this._counts[type] || 0) + 1
    this._counts[type] = number
    let id = type[0].toLowerCase() + type.substring(1) + number
    _register(id, this.url, type, instance)

    return Promise.resolve({id, instance})
  }

  /**
   * Create an execution context for a particular language
   */
  createContext(language) {
    const context = this._contexts[language]
    if (context) return context
    else {
      const type = {
        'js': 'JavascriptContext',
        'mini': 'MiniContext',
        'node': 'NodeContext',
        'py': 'PythonContext',
        'pyjp': 'JupyterContext',
        'r': 'RContext',
        'sql': 'SqliteContext'
      }[language]

      const options = {
        'pyjp': {
          language: 'python'
        }
      }[language] || {}

      if (!type) {
        return Promise.reject(new Error(`Unable to create an execution context for language ${language}`))
      } else {
        const promise = this.create(type, options).then((result) => {
          if (result instanceof Error) {
            // Unable to create so set the cached context promise to null
            // so a retry is performed next time this method is called
            // (at which time another peer that provides the context may be available)
            this._contexts[language] = null
            return result
          } else {
            // Get a list of fuctions from the context so that `FunctionManager` can
            // dispatch a `call` operation to the context if necessary. Implemented
            // optimistically i.e. will not fail if the context does not implement `libraries`
            const context = result.instance
            if (typeof context.libraries === 'function') {
              return context.libraries().then((libraries) => {
                this._functionManager.importLibraries(context, libraries)
                return context
              }).catch((error) => {
                console.log(error) // eslint-disable-line
              })
            } else {
              return context
            }
          }
        })
        this._contexts[language] = promise
        return promise
      }
    }
  }

  _get(host, path) {
    const token = this._token(host)
    return this._request('GET', host + path, null, token)
  }

  _post(host, path, data) {
    const token = this._token(host)
    return this._request('POST', host + path, data, token)
  }

  _put(host, path, data) {
    const token = this._token(host)
    return this._request('PUT', host + path, data || {}, token)
  }

  _delete(host, path) {
    const token = this._token(host)
    return this._request('DELETE', host + path, null, token)
  }

  _token(url) {
    const host = this._hosts.get(url)
    if (!host) return
    const key = host.key
    if (!key) return
    const iat = Math.round(Date.now() / 1000)
    const hid = this._id
    const seq = host.sent + 1
    host.sent = seq
    const payload = { iat, hid, seq }
    const token = KJUR.jws.JWS.sign('HS256', '{"alg":"HS256","typ":"JWT"}', payload, {rstr: key})
    return token
  }

  _request (method, url, data, token) {
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

      if (token) {
        request.setRequestHeader('Authorization', 'Bearer ' + token)
      }

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
}
