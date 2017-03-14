import test from 'tape'

import getSandbox from '../getSandbox'
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