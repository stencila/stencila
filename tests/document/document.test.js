// Integration tests for src/document/document

import test from 'tape'

import getSandbox from '../getSandbox'
import { DocumentPage } from '../../index.es'

test('Document: mounting a DocumentPage', (t) => {
  const sandbox = getSandbox(t)
  DocumentPage.mount({
    backend: new MemoryBackend(),
    archiveURL: '/tests/documents/simple/default.html'
  }, sandbox)
  t.end()
})