import { JATSImporter, TextureConfigurator } from 'substance-texture'
import { ArticlePackage, ArticlePage, Host, getQueryStringParam, FunctionManager} from 'stencila'

window.addEventListener('load', () => {
  const example = getQueryStringParam('example') || 'blank'

  let peers = (getQueryStringParam('peers') || window.STENCILA_PEERS)
  if (peers) peers = peers.split(',')

  const discover = window.STENCILA_DISCOVER ? parseFloat(window.STENCILA_DISCOVER) : false

  let functionManager = new FunctionManager()
  functionManager.importLibrary('core', window.STENCILA_LIBCORE)

  let host = new Host({
    functionManager,
    peers: peers,
    discover: discover,
  })
  host.initialize().then(() => {
    let xml = window.vfs.readFileSync(`examples/article/${example}.xml`)
    let jatsImporter = new JATSImporter()
    let jats = jatsImporter.import(xml)
    let configurator = new TextureConfigurator()
    configurator.import(ArticlePackage)
    const articleImporter = configurator.createImporter('texture-jats')
    const article = articleImporter.importDocument(jats.dom)
    ArticlePage.mount({ article, configurator, host }, window.document.body)

    window.stencila = { host, article }
  })
})
