import test from 'tape'

import getSandbox from '../getSandbox'
import { DocumentPage } from '../../index.es'
import TestBackend from '../backend/TestBackend'
import { isNil } from 'substance'

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

function wait(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms)
  })
}
