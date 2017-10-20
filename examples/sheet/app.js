import { Configurator } from 'substance'
import { SheetPackage, SheetPage, SheetSchema, Host, getQueryStringParam} from 'stencila'

import blank from './blank'
import fullup from './fullup'

window.addEventListener('load', () => {
  const example = getQueryStringParam('example') || 'blank'

  let peers = (getQueryStringParam('peers') || window.STENCILA_PEERS)
  if (peers) peers = peers.split(',')

  const discover = window.STENCILA_DISCOVER ? parseFloat(window.STENCILA_DISCOVER) : false

  let host = new Host({
    peers: peers,
    discover: discover,
  })
  host.initialize().then(() => {
    let configurator = new Configurator()
    configurator.import(SheetPackage)
    const importer = configurator.createImporter(SheetSchema.getName())
    
    let generator = {
      blank: blank,
      fullup: fullup
    }[example]
    if (!generator) console.error('No such example: ' + example)
    const xml = generator()

    const sheet = importer.importDocument(xml)
    SheetPage.mount({ sheet, host }, window.document.body)
  })
})
