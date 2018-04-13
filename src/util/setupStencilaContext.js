import Host from '../host/Host'
import getQueryStringParam from '../util/getQueryStringParam'
import Engine from '../engine/Engine'

export default function setupStencilaContext() {
  // Get configuration options from environment variables and query parameters
  const libs = {
    //core: window.STENCILA_LIBCORE
  }
  // Stencila Host (for requesting external execution contexts etc)
  let hosts = []
  // Use the origin as a remote Stencila Host?
  if (window.STENCILA_ORIGIN_HOST) {
    hosts.push(window.location.origin)
  }
  // List of any other remote Stencila Hosts
  // Deprecated `peers` configuration option (hosts seems like a less confusing name)
  const hostsExtra = (
    getQueryStringParam('hosts') || window.STENCILA_HOSTS ||
    getQueryStringParam('peers') || window.STENCILA_PEERS
  )
  if (hostsExtra) hosts = hosts.concat(hostsExtra.split(','))
  // Try to discover hosts on http://127.0.0.1?
  const discover = parseFloat(getQueryStringParam('discover') || window.STENCILA_DISCOVER || '-1')
  // Instantiate and initialise the host
  const engine = new Engine()
  const host = new Host({libs, hosts, discover, engine })
  engine.setHost(host)
  const functionManager = host._functionManager

  return { host, functionManager, engine}
}
