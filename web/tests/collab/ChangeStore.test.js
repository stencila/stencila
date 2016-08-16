'use strict';

// Tests based (heavily!) on Substance `testChangeStore.js`

var test = require('tape');

// Don't run in browser
if (typeof window === 'undefined') {

  var mockery = require('mockery');
  mockery.registerMock('redis', require('fakeredis'));
  mockery.enable({
    warnOnReplace: false,
    warnOnUnregistered: false
  });

  var ChangeStore = require('../../collab/ChangeStore');
  var store = new ChangeStore();

  test('ChangeStore.addChange', function(t) {

    store.addChange({
      documentId: 'test-doc-1',
      change: {
        ops: [{some: 'operation'}]
      }
    }, function(err, version) {
      t.equal(err, null);
      t.equal(version, 1);

      store.getChanges({
        documentId: 'test-doc-1',
        sinceVersion: 0
      }, function(err, result) {
        t.equal(err, null);
        t.equal(result.changes.length, 1);
        t.deepEqual(result.changes,[{"ops":[{"some":"operation"}]}]);
        t.equal(result.version, 1);
        t.end();
      });
    });

    store.addChange({
      documentId: 'test-doc-2',
      change: {
        ops: [{some: 'operation'}]
      }
    }, function(err, version) {
      t.equal(err, null);
      t.equal(version, 1);
    });

  });

  test("ChangeStore.getChanges Return changes of test-doc-1", function(t) {
    var args = {
      documentId: 'test-doc-1',
      sinceVersion: 0
    };
    store.getChanges(args, function(err, result) {
      t.notOk(err, 'Should not error');
      t.equal(result.changes.length, 1, 'Should be only one change');
      t.equal(result.version, 1, 'Document version should be 1');
      t.end();
    });
  });

  test("ChangeStore.getChanges Return all changes of test-doc-2 by not specifying sinceVersion", function(t) {
    var args = {
      documentId: 'test-doc-2'
    };
    store.getChanges(args, function(err, result) {
      t.notOk(err, 'Should not error');
      t.equal(result.changes.length, 1, 'Should be only one change');
      t.equal(result.version, 1, 'Document version should be 1');
      t.end();
    });
  });

  test("ChangeStore.getChanges Should return no changes if sinceVersion = actual version", function(t) {
    var args = {
      documentId: 'test-doc-2',
      sinceVersion: 1
    };
    store.getChanges(args, function(err, result) {
      t.notOk(err, 'Should not error');
      t.equal(result.changes.length, 0, 'Should have zero changes');
      t.equal(result.version, 1, 'Document version should be 1');
      t.end();
    });
  });

  test("ChangeStore.getChanges Return changes of test-doc-2 between version 1 and version 2", function(t) {
    // Add two changes
    store.addChange({
      documentId: 'test-doc-2',
      change: {
        ops: [{some: 'operation2'}],
      }
    }, function(err, result) {
      t.equal(result, 2, 'Latest version should be 2');
      store.addChange({
        documentId: 'test-doc-2',
        change: {
          ops: [{some: 'operation3'}],
        }
      }, function(err, result) {
        t.equal(result, 3, 'Latest version should be 3');
        var args = {
          documentId: 'test-doc-2',
          sinceVersion: 1,
          toVersion: 2
        };
        store.getChanges(args, function(err, result) {
          t.notOk(err, 'Should not error');
          t.equal(result.changes.length, 1, 'Should be only one change');
          t.equal(result.changes[0].ops[0].some, 'operation2', 'Should be correct operation');
          t.equal(result.version, 3, 'Latest version should be 3');
          t.end();
        });
      });
    });
  });

  test("ChangeStore.getChanges Invalid use of getChanges sinceVersion argument", function(t) {
    var args = {
      documentId: 'test-doc-1',
      sinceVersion: -5
    };
    store.getChanges(args, function(err) {
      t.equal(err.name, 'ChangeStore.ReadError', 'Should give a read error as invalid version provided');
      t.end();
    });
  });

  test("ChangeStore.getChanges Invalid use of getChanges toVersion argument", function(t) {
    var args = {
      documentId: 'test-doc-1',
      toVersion: -3
    };
    store.getChanges(args, function(err) {
      t.equal(err.name, 'ChangeStore.ReadError', 'Should give a read error as invalid version provided');
      t.end();
    });
  });

  test("ChangeStore.getChanges Invalid use of getChanges version arguments", function(t) {
    var args = {
      documentId: 'test-doc-1',
      sinceVersion: 2,
      toVersion: 1
    };
    store.getChanges(args, function(err) {
      t.equal(err.name, 'ChangeStore.ReadError', 'Should give a read error as invalid version provided');
      t.end();
    });
  });


  test("ChangeStore.getVersion should return version", function(t) {
    store.getVersion('test-doc-1', function(err, version) {
      t.equal(err, null);
      t.equal(version, 1);
      t.end();
    });
  });

  test("ChangeStore.getVersion should return version==0 if no changes are found", function(t) {
    store.getVersion('not-existing-doc', function(err, version) {
      t.equal(err, null);
      t.equal(version, 0, 'Document version should equal 0');
      t.end();
    });
  });


  test("ChangeStore.deleteChanges of test doc", function(t) {
    store.deleteChanges('test-doc-1', function(err, changeCount) {
      t.notOk(err, 'Should not error');
      t.equal(changeCount, 1, 'There should be 1 deleted change');
      store.getChanges({
        documentId: 'test-doc-1',
        sinceVersion: 0
      }, function(err, result) {
        t.notOk(err, 'Should not error');
        t.equal(result.changes.length, 0, 'There should not be changes anymore');
        t.equal(result.version, 0, 'Document version should be 0');
        t.end();
      });
    });
  });

  test("ChangeStore.deleteChanges of not existing doc", function(t) {
    store.deleteChanges('not-existing-doc', function(err, changeCount) {
      t.notOk(err, 'Should not error');
      t.equal(changeCount, 0, 'There should be 0 deleted changes');
      t.end();
    });
  });

}
