import test from 'tape'
import { isNil } from 'substance'
import { spy, wait, getSandbox } from '../testHelpers'
import { DocumentPage } from '../../index.es'
import TestBackend from '../backend/TestBackend'

// Integration tests for src/document

test('Document: mounting a DocumentPage', (t) => {
  const sandbox = getSandbox(t)
  const page = DocumentPage.mount({
    backend: new TestBackend(),
    archiveURL: '/tests/documents/simple/default.html'
  }, sandbox)
  t.ok(page.isMounted(), 'DocumentPage should be mounted')
  t.end()
})

test('Document: switching documents', (t) => {
  const sandbox = getSandbox(t)
  const page = DocumentPage.mount({
    backend: new TestBackend(),
    archiveURL: '/tests/documents/simple/default.html'
  }, sandbox)
  t.plan(2)
  Promise.resolve()
  .then(() => {
    page.extendProps({
      archiveURL: '/tests/documents/simple/default.html'
    })
  })
  .then(wait(10))
  .then(() => {
    let simple = page.find('[data-id=simple]')
    t.notOk(isNil(simple), 'Element #simple should be on the page.')
  })
  .then(() => {
    page.extendProps({
      archiveURL: '/tests/documents/paragraph/default.html'
    })
  })
  .then(wait(10))
  .then(() => {
    let paragraph = page.find('[data-id=paragraph]')
    t.notOk(isNil(paragraph), 'Element #paragraph should be on the page.')
  })
})


test('Document: open all test documents', (t) => {
  const testBackend = new TestBackend()
  const docUrls = testBackend._getDocumentUrls()
  const CHECKS_PER_URL = 1
  t.plan(docUrls.length*CHECKS_PER_URL)

  const sandbox = getSandbox(t)
  const page = DocumentPage.mount({
    backend: testBackend,
    archiveURL: '/tests/documents/simple/default.html'
  }, sandbox)

  let p = Promise.resolve()
  for (let i = 0; i < docUrls.length; i++) {
    const url = docUrls[i]
    p = p.then(()=>{
      page.extendProps({
        archiveURL: url
      })
    })
    .then(wait(10))
    .then(() => {
      t.notOk(isNil(page.state.editorSession), `Page should have opened ${url}`)
    })
  }
})

test('Document: saving archive', (t) => {
  t.plan(1)

  const sandbox = getSandbox(t)
  const backend = new TestBackend()
  const archiveURL = '/tests/documents/simple/default.html'
  const page = DocumentPage.mount({ backend, archiveURL }, sandbox)
  const archive = backend.getArchive(archiveURL)
  const _writeFile = spy(archive, 'writeFile')
  Promise.resolve()
  .then(() => {
    page.save()
  })
  .then(wait(10))
  .then(() => {
    t.equal(_writeFile.callCount, 1, 'archive.writeFile should have been called.')
  })
})
