import test from 'tape'

import { getSandbox } from '../testHelpers'
import { Dashboard } from '../../index.es'
import TestBackend from '../backend/TestBackend'

// Integration tests for src/dashboard

test('Dashboard: mounting a Dashboard', (t) => {
  const sandbox = getSandbox(t)
  const dashboard = Dashboard.mount({
    backend: new TestBackend(),
    resolveEditorURL() {
      return "javascript:void(0)" // eslint-disable-line
    }
  }, sandbox)
  t.ok(dashboard.isMounted(), 'Dashboard should be mounted')
  t.end()
})
