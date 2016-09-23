'use strict';

// Tests based (heavily!) on Substance `testDocumentStore.js`

import test from 'tape'

// Don't run in browser
if (typeof window === 'undefined') {
  require('../helpers/mockery')

  var DocumentStore = require('../../collab/DocumentStore')
  var store = new DocumentStore();

  test('DocumentStore.createDocument', function (t) {
    var newDoc = {
      documentId: 'test-doc',
      schemaName: 'stencila-document',
      schemaVersion: '1.0.0',
      version: 1,
      info: {
        custom: 'some custom data'
      }
    };
    store.createDocument(newDoc, function (err, doc) {
      t.notOk(err);
      t.ok(doc, 'valid doc entry expected');
      t.equal(doc.schemaName, 'stencila-document', 'schemaName should be "stencila-document"');
      t.end();
    });
  });

  test('DocumentStore.createDocument without providing a documentId', function (t) {
    var newDoc = {
      schemaName: 'stencila-document',
      schemaVersion: '1.0.0'
    };
    store.createDocument(newDoc, function (err, doc) {
      t.notOk(err, 'should not error');
      t.ok(doc, 'valid doc entry expected');
      t.ok(doc.documentId, 'Auto-generated documentId should be returned');
      t.end();
    });
  });

  test('DocumentStore.createDocument that already exists', function (t) {
    var newDoc = {
      documentId: 'test-doc',
      schemaName: 'stencila-document',
      schemaVersion: '1.0.0'
    };
    store.createDocument(newDoc, function (err, doc) {
      t.ok(err, 'should error');
      t.equal(err.name, 'DocumentStore.CreateError', 'Should give a create error');
      t.notOk(doc, 'doc should be undefined');
      t.end();
    });
  });

  test('DocumentStore.getDocument', function (t) {
    store.getDocument('test-doc', function (err, doc) {
      t.notOk(err, 'should not error');
      t.ok(doc, 'doc data expected');
      t.equal(doc.documentId, 'test-doc', 'documentId should be "test-doc"');
      t.equal(doc.schemaName, 'stencila-document', 'schemaName should be stencila-document');
      t.equal(doc.schemaVersion, '1.0.0', 'schemaVersion should be 1.0.0');
      t.equal(doc.version, 1, 'doc version should be 1');
      t.end();
    });
  });

  test('DocumentStore.getDocument Get document that does not exist', function (t) {
    store.getDocument('not-there-doc', function (err) {
      t.ok(err, 'should error');
      t.equal(err.name, 'DocumentStore.ReadError', 'Should give a read error for deleted document');
      t.end();
    });
  });

  test('DocumentStore.getDocument Update a document', function (t) {
    var updateProps = {
      schemaName: 'stencila-sheet',
      schemaVersion: '2.0.0'
    };
    store.updateDocument('test-doc', updateProps, function (err, doc) {
      t.notOk(err, 'There should be no error');
      t.ok(doc, 'valid doc entry expected');
      t.equal(doc.schemaName, 'stencila-sheet', 'schemaName should be "stencila-sheet" after update');
      t.equal(doc.schemaVersion, '2.0.0', 'schemaVersion should be "2.0.0" after update');
      t.end();
    });
  });

  test('DocumentStore.getDocument Update a document that does not exist', function (t) {
    store.updateDocument('doc-x', {schemaName: 'blog-article'}, function (err, doc) {
      t.ok(err, 'should error');
      t.equal(err.name, 'DocumentStore.UpdateError', 'should return an update error.');
      t.notOk(doc, 'doc should be undefined');
      t.end();
    });
  });

  test('DocumentStore.deleteDocument', function (t) {
    store.deleteDocument('test-doc', function (err, doc) {
      t.notOk(err, 'There should be no error');
      t.ok(doc, 'Deleted doc entry should be returned');
      t.equal(doc.schemaName, 'stencila-sheet', 'doc schemaName should be "stencila-sheet"');

      store.getDocument('test-doc', function (err, doc) {
        t.ok(err, 'should error');
        t.equal(err.name, 'DocumentStore.ReadError', 'Should give a read error for deleted document');
        t.notOk(doc, 'doc should be undefined');
        t.end();
      });
    });
  });

  test('DocumentStore.deleteDocument that does not exist', function (t) {
    store.deleteDocument('doc-x', function (err, doc) {
      t.ok(err, 'should error');
      t.equal(err.name, 'DocumentStore.DeleteError', 'Should give a delete error');
      t.notOk(doc, 'doc should be undefined');
      t.end();
    });
  });

  test('DocumentStore.documentExists should return true for existing document', function (t) {
    store.createDocument({
      documentId: 'test-sheet',
      schemaName: 'stencila-sheet',
      schemaVersion: '1.0.0',
      version: 1
    }, function (err, doc) {
      t.notOk(err, 'Should not error');
      store.documentExists('test-sheet', function (err, exists) {
        t.notOk(err, 'Should not error');
        t.ok(exists, 'Exists should be true');
        t.end();
      });
    });
  });

  test('DocumentStore.documentExists should return false for non-existing document', function (t) {
    store.documentExists('not-existing-doc', function (err, exists) {
      t.notOk(err, 'Should not error');
      t.notOk(exists, 'Exists should be false');
      t.end();
    });
  });
}
