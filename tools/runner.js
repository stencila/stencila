/**
 * A Stencila document runner for use in integration tests and in 
 * continuous integration tools.
 *
 * Executes all cells in a document, outputs execution information and
 * exits with the number of cells with errors
 */

const fs = require('fs')
const stencila = require('..')
const substance = require('substance')

const file = process.argv[2]
if (!file) {
  console.error('No file path given')
  process.exit(1)
}

// Host for getting external execution contexts
const host = new stencila.Host({
  peers: process.env.STENCILA_PEERS ? process.env.STENCILA_PEERS.split(' ') : [],
  discover: process.env.STENCILA_DISCOVER ? parseFloat(process.env.STENCILA_DISCOVER) : false,
})

// Backend for getting document content
const backend = new stencila.MemoryBackend({
  'file': fs.readFileSync(file, {encoding: 'utf8'})
})

// DOM for mounting
const document = substance.DefaultDOMElement.parseHTML('<html><body></body></html>')
const body = document.find('body')

const documentPage = stencila.DocumentPage.mount({
  host: host,
  backend: backend,
  documentId: 'file'
}, body)

const start = process.hrtime()

// Give time to compute
// TODO: better way to do this! listen to some 'finished' event?
setTimeout(() => {
  // Record duration
  const diff = process.hrtime(start)
  const duration = diff[0] + diff[1]/1e9

  // Check each cell for errors
  let counts = {
    cells: 0,
    errors: 0
  }
  let errors = []
  const cells = documentPage.state.cellEngine._cells
  for (let key of Object.keys(cells)) {
    let cell = cells[key]
    if (cell.hasErrors()) {
      if (cell.hasSyntaxError()) {
        errors.push({
          where: key,
          type: "syntax",
          message: cell.getSyntaxError()
        })
      }
      if (cell.hasRuntimeErrors()) {
        errors.push({
          where: key,
          type: "runtime",
          message: cell.getRuntimeErrors()
        })
      }
      counts.errors += 1
    }
    counts.cells += 1
  }

  // Output results for use by other tools
  console.log(JSON.stringify({
    file,
    counts,
    duration,
    errors
  }))

  // Exit with the number of errors
  process.exit(errors)
}, 1000)
