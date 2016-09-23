'use strict'

// Tests based (heavily!) on Substance `testSnapshotStore.js`

import test from 'tape'

// Don't run in browser
if (typeof window === 'undefined') {
  require('../helpers/mockery')

  var SnapshotStore = require('../../collab/SnapshotStore').default
  var store = new SnapshotStore()

  test('SnapshotStore.saveSnapshot Store a snapshot', function (t) {
    store.saveSnapshot({
      documentId: 'test-doc-1',
      version: 1,
      data: {some: 'snapshot'}
    }, function (err, snapshot) {
      t.notOk(err, 'Should not error')
      t.ok(snapshot, 'Stored snapshot entry expected')

      store.saveSnapshot({
        documentId: 'test-doc-2',
        version: 3,
        data: {some: 'snapshot'}
      }, function (err, snapshot) {
        t.notOk(err, 'Should not error')
        t.end()
      })
    })
  })

  test('SnapshotStore.getSnapshot Retrieve snapshot for test-doc-1', function (t) {
    store.getSnapshot({
      documentId: 'test-doc-1'
    }, function (err, snapshot) {
      t.notOk(err, 'Should not error')
      t.equal(snapshot.version, 1, 'Retrieved version should be 1')
      t.ok(snapshot.data, 'Snapshot should have some data')
      t.ok(snapshot.documentId, 'Snapshot should have the documentId')
      t.end()
    })
  })

  test('SnapshotStore.getSnapshot Retrieve snapshot for test-doc-1 with version=1', function (t) {
    store.getSnapshot({
      documentId: 'test-doc-1',
      version: 1
    }, function (err, snapshot) {
      t.notOk(err, 'Should not error')
      t.equal(snapshot.version, 1, 'Retrieved version should be 1')
      t.ok(snapshot.data, 'Snapshot should have some data')
      t.ok(snapshot.documentId, 'Snapshot should have the documentId')
      t.end()
    })
  })

  test('SnapshotStore.getSnapshot Retrieve snapshot for test-doc-2', function (t) {
    store.getSnapshot({
      documentId: 'test-doc-2'
    }, function (err, snapshot) {
      t.notOk(err, 'Should not error')
      t.equal(snapshot.version, 3, 'Retrieved version should be 3')
      t.ok(snapshot.data, 'Snapshot should have some data')
      t.ok(snapshot.documentId, 'Snapshot should have the documentId')
      t.end()
    })
  })

  test('SnapshotStore.getSnapshot Retrieve snapshot for test-doc-2 with version=2', function (t) {
    store.getSnapshot({
      documentId: 'test-doc-2',
      version: 2
    }, function (err, snapshot) {
      t.notOk(err, 'Should not error')
      t.notOk(snapshot, 'Snapshot should be undefined')
      t.end()
    })
  })

  test('SnapshotStore.getSnapshot Retrieve snapshot for test-doc-2 with version=3', function (t) {
    store.getSnapshot({
      documentId: 'test-doc-2',
      version: 3
    }, function (err, snapshot) {
      t.notOk(err, 'Should not error')
      t.equal(snapshot.version, 3, 'Retrieved version should be 3')
      t.ok(snapshot.data, 'Snapshot should have some data')
      t.ok(snapshot.documentId, 'Snapshot should have the documentId')
      t.end()
    })
  })
}
