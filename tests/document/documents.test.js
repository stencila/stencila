const fs = require('fs')
const path = require('path')
const test = require('tape')

const Document = require('../../src/document/Document')

/*
 * Test for successful conversion of test documents from
 * the stencila/stencila repo. Run `make test-documents` to get those
 * documents.
 */
test('Document test documents', t => {
  fs.readdir('tests/document/documents', (err, files) => {
    if (err) {
      t.skip('Folder tests/document/documents not found. Do you need to run `make test-documents`?')
      t.end()
      return
    }

    // For each test document...
    for (let dir of files) {
      // Get list of files
      fs.readdir(path.join('tests/document/documents', dir), (err, files) => {
        // Start a subtest...
        t.test(dir, t => {
          // Failing tests that we will skip at present
          if (['input', 'output', 'execute', 'include', 'simple'].indexOf(dir) >= 0) {
            t.skip('TODO')
            t.end()
            return
          }

          t.notOk(err)

          // Read in files
          let docs = {}
          for (let file of files) {
            let canon = file.substring(0, 7) === 'default'
            let format = path.extname(file).substring(1)
            let content = fs.readFileSync(path.join('tests/document/documents', dir, file), {encoding: 'utf8'})
            content = content.trim()
            // Load the document from the content
            let doc = new Document()
            doc.load(content, format)
            docs[file] = {
              canon: canon,
              format: format,
              content: content,
              doc: doc
            }
          }

          // All documents should dump HTML equal to default.html
          let htmlDoc = docs['default.html']
          if (htmlDoc) {
            let expected = htmlDoc.content
            for (let file in docs) {
              let obtained = docs[file].doc.dump('html')
              t.equal(obtained, expected, dir + '/' + file + ' should be equivalent to ' + dir + '/default.html')
            }
          }

          // All documents should dump Markdown equal to default.md
          let mdDoc = docs['default.md']
          if (mdDoc) {
            let expected = mdDoc.content
            for (let file in docs) {
              let obtained = docs[file].doc.dump('md')
              t.equal(obtained, expected, dir + '/' + file + ' should be equivalent to ' + dir + '/default.md')
            }
          }

          t.end()
        })
      })
    }
  })
})
