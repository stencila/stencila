/**
 * A Stencila document runner for use in integration tests and in 
 * continuous integration tools.
 *
 * Executes all cells in a document, outputs execution information and
 * exits with the number of cells with errors
 */

const crypto = require('crypto')
const fs = require('fs')
const path = require('path')
const stencila = require('..')
const substance = require('substance')

const file = process.argv[2]
if (!file) {
  console.error('No file path given')
  process.exit(1)
}

let content = fs.readFileSync(file, {encoding: 'utf8'})

// Convert content to internal HTML
let {ext} = path.parse(file)
let Converter = {
  '.md': stencila.DocumentMarkdownConverter,
  '.ipynb': stencila.DocumentJupyterConverter,
  '.rmd': stencila.DocumentRMarkdownConverter
}[ext.toLowerCase()]
let html
if (Converter) {
  html = `
<!DOCTYPE html>
<html>
  <head>
    <meta name="viewport" content="width=device-width, initial-scale=1">
  </head>
  <body>
    <main>
      <div id="data" data-format="html">
        <div class="content">${new Converter().importContent(content)}</div>
      </div>
    </main>
  </body>
</html>`
} else html = content

// Host for getting external execution contexts
const host = new stencila.Host({
  peers: process.env.STENCILA_PEERS ? process.env.STENCILA_PEERS.split(' ') : [],
  discover: process.env.STENCILA_DISCOVER ? parseFloat(process.env.STENCILA_DISCOVER) : false,
})

// Backend for getting document content
const backend = new stencila.MemoryBackend({
  'file': html
})

// DOM for mounting
const document = substance.DefaultDOMElement.parseHTML('<html><body></body></html>')
const body = document.find('body')

const documentPage = stencila.DocumentPage.mount({
  host: host,
  backend: backend,
  documentId: 'file'
}, body)

// Function for getting HTML version of document content
function documentHtml () {
  let doc = documentPage.state.editorSession.getDocument()
  return stencila.documentConversion.exportHTML(doc)
}

// Function for getting a digest of document content
function documentDigest () {
  let sha256 = crypto.createHash('sha256')
  sha256.update(documentHtml())
  return sha256.digest('hex')
}

// Get digest before the document is updated
// TODO: documentDigest() does not work here, needs to be triggered
// in response to an event
const digestPre = null 

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
    errors,
    digests: {
      pre: digestPre,
      post: documentDigest()
    }
  }))

  // Exit with the number of errors
  process.exit(errors)
}, 1000)
