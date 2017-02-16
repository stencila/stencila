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
  // Get list of test document directories
  try {
    var dirs = fs.readdirSync('tests/document/documents')
  } catch (error) {
    t.skip('Folder tests/document/documents not found. Do you need to run `make test-documents`?')
    t.end()
    return
  }

  // For each test document...
  for (let dir of dirs) {
    // Start a subtest...
    t.test(dir, st => {
      // Failing tests that we will skip at present
      if (['execute', 'simple'].indexOf(dir) >= 0) {
        st.skip('TODO')
        st.end()
        return
      }

      // Get list of files
      let files = fs.readdirSync(path.join('tests/document/documents', dir))

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
          st.equal(obtained, expected, dir + '/' + file + ' should be equivalent to ' + dir + '/default.html')
        }
      }

      // All documents should dump Markdown equal to default.md
      let mdDoc = docs['default.md']
      if (mdDoc) {
        let expected = mdDoc.content
        for (let file in docs) {
          let obtained = docs[file].doc.dump('md')
          st.equal(obtained, expected, dir + '/' + file + ' should be equivalent to ' + dir + '/default.md')
        }
      }

      st.end()
    })
  }
  t.end()
})
