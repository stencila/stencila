import Host from '../host/Host'
import ArticleEngineAdapter from '../article/ArticleEngineAdapter'
import SheetEngineAdapter from '../sheet/SheetEngineAdapter'
import getQueryStringParam from '../util/getQueryStringParam'

export default function setupStencilaContext(documentContainer) {
  // Get configuration options from environment variables and query parameters
  const libs = {
    core: window.STENCILA_LIBCORE
  }
  let peers = (
    // Deprecated `peers` configuration option (hosts seems like a less confusing name)
    getQueryStringParam('hosts') || window.STENCILA_HOSTS ||
    getQueryStringParam('peers') || window.STENCILA_PEERS
  )
  if (peers) peers = peers.split(',')
  const discover = (getQueryStringParam('discover') || window.STENCILA_DISCOVER) === 'true'

  // Instantiate and initialise the host
  let host = new Host({libs, peers, discover})
  return host.initialize().then(() => {
    let docEntries = documentContainer.getDocumentEntries()
    docEntries.forEach((entry) => {
      let editorSession = documentContainer.getEditorSession(entry.id)
      if (entry.type === 'article') {
        let engineAdapter = new ArticleEngineAdapter(editorSession)
        engineAdapter.connect(host.engine, { id: entry.id })
      } else if (entry.type === 'sheet') {
        let engineAdapter = new SheetEngineAdapter(editorSession)
        engineAdapter.connect(host.engine, { id: entry.id })
      }
    })
    return { 
      host, 
      functionManager: host.functionManager, 
      engine: host.engine 
    }
  })
}