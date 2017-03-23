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
    documentId: '/tests/documents/simple/default.html'
  }, sandbox)
  t.ok(page.isMounted(), 'DocumentPage should be mounted')
  t.end()
})

test('Document: switching documents', (t) => {
  const sandbox = getSandbox(t)
  const page = DocumentPage.mount({
    backend: new TestBackend(),
    documentId: '/tests/documents/simple/default.html'
  }, sandbox)
  t.plan(2)
  Promise.resolve()
  .then(() => {
    page.extendProps({
      documentId: '/tests/documents/simple/default.html'
    })
  })
  .then(wait(10))
  .then(() => {
    let simple = page.find('[data-id=simple]')
    t.notOk(isNil(simple), 'Element #simple should be on the page.')
  })
  .then(() => {
    page.extendProps({
      documentId: '/tests/documents/paragraph/default.html'
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
  const docIds = testBackend._getDocumentIds()
  const CHECKS_PER_URL = 1
  t.plan(docIds.length*CHECKS_PER_URL)


  const sandbox = getSandbox(t)
  const page = DocumentPage.mount({
    backend: testBackend,
    documentId: '/tests/documents/simple/default.html'
  }, sandbox)

  let p = Promise.resolve()
  for (let i = 0; i < docIds.length; i++) {
    const docId = docIds[i]
    p = p.then(()=> {
      page.extendProps({
        documentId: docId
      })
    })
    .then(wait(10))
    .then(() => {
      t.notOk(isNil(page.state.editorSession), `Page should have opened ${docId}`)
    })
  }
})

test('Document: storing buffer', (t) => {
  t.plan(1)

  const sandbox = getSandbox(t)
  const backend = new TestBackend()
  const documentId = '/tests/documents/simple/default.html'
  const page = DocumentPage.mount({ backend, documentId }, sandbox)
  let _storeBuffer
  backend.getBuffer(documentId)
  .then(() => {
    _storeBuffer = spy(backend, 'storeBuffer')
  })
  .then(() => {
    page.save()
  })
  .then(wait(10))
  .then(() => {
    t.equal(_storeBuffer.callCount, 1, 'backend._storeBuffer should have been called.')
  })
})
