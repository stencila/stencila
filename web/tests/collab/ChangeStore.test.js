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

  test('ChangeStore.addChange', function(assert) {

    store.addChange({
      documentId: 'test-doc-1',
      change: {
        ops: [{some: 'operation'}]
      }
    }, function(err, version) {
      assert.equal(err, null);
      assert.equal(version, 1);

      store.getChanges({
        documentId: 'test-doc-1',
        sinceVersion: 0
      }, function(err, result) {
        assert.equal(err, null);
        assert.equal(result.changes.length, 1);
        assert.deepEqual(result.changes,[{"ops":[{"some":"operation"}]}]);
        assert.equal(result.version, 1);
        assert.end();
      });
    });

    store.addChange({
      documentId: 'test-doc-2',
      change: {
        ops: [{some: 'operation'}]
      }
    }, function(err, version) {
      assert.equal(err, null);
      assert.equal(version, 1);
    });

  });

  test("ChangeStore.getChanges Return changes of test-doc-1", function(assert) {
    var args = {
      documentId: 'test-doc-1',
      sinceVersion: 0
    };
    store.getChanges(args, function(err, result) {
      assert.notOk(err, 'Should not error');
      assert.equal(result.changes.length, 1, 'Should be only one change');
      assert.equal(result.version, 1, 'Document version should be 1');
      assert.end();
    });
  });

  test("ChangeStore.getChanges Return all changes of test-doc-2 by not specifying sinceVersion", function(assert) {
    var args = {
      documentId: 'test-doc-2'
    };
    store.getChanges(args, function(err, result) {
      assert.notOk(err, 'Should not error');
      assert.equal(result.changes.length, 1, 'Should be only one change');
      assert.equal(result.version, 1, 'Document version should be 1');
      assert.end();
    });
  });

  test("ChangeStore.getChanges Should return no changes if sinceVersion = actual version", function(assert) {
    var args = {
      documentId: 'test-doc-2',
      sinceVersion: 1
    };
    store.getChanges(args, function(err, result) {
      assert.notOk(err, 'Should not error');
      assert.equal(result.changes.length, 0, 'Should have zero changes');
      assert.equal(result.version, 1, 'Document version should be 1');
      assert.end();
    });
  });

  test("ChangeStore.getChanges Return changes of test-doc-2 between version 1 and version 2", function(assert) {
    // Add two changes
    store.addChange({
      documentId: 'test-doc-2',
      change: {
        ops: [{some: 'operation2'}],
      }
    }, function(err, result) {
      assert.equal(result, 2, 'Latest version should be 2');
      store.addChange({
        documentId: 'test-doc-2',
        change: {
          ops: [{some: 'operation3'}],
        }
      }, function(err, result) {
        assert.equal(result, 3, 'Latest version should be 3');
        var args = {
          documentId: 'test-doc-2',
          sinceVersion: 1,
          toVersion: 2
        };
        store.getChanges(args, function(err, result) {
          assert.notOk(err, 'Should not error');
          assert.equal(result.changes.length, 1, 'Should be only one change');
          assert.equal(result.changes[0].ops[0].some, 'operation2', 'Should be correct operation');
          assert.equal(result.version, 3, 'Latest version should be 3');
          assert.end();
        });
      });
    });
  });

  test("ChangeStore.getChanges Invalid use of getChanges sinceVersion argument", function(assert) {
    var args = {
      documentId: 'test-doc-1',
      sinceVersion: -5
    };
    store.getChanges(args, function(err) {
      assert.equal(err.name, 'ChangeStore.ReadError', 'Should give a read error as invalid version provided');
      assert.end();
    });
  });

  test("ChangeStore.getChanges Invalid use of getChanges toVersion argument", function(assert) {
    var args = {
      documentId: 'test-doc-1',
      toVersion: -3
    };
    store.getChanges(args, function(err) {
      assert.equal(err.name, 'ChangeStore.ReadError', 'Should give a read error as invalid version provided');
      assert.end();
    });
  });

  test("ChangeStore.getChanges Invalid use of getChanges version arguments", function(assert) {
    var args = {
      documentId: 'test-doc-1',
      sinceVersion: 2,
      toVersion: 1
    };
    store.getChanges(args, function(err) {
      assert.equal(err.name, 'ChangeStore.ReadError', 'Should give a read error as invalid version provided');
      assert.end();
    });
  });


  test("ChangeStore.getVersion should return version", function(assert) {
    store.getVersion('test-doc-1', function(err, version) {
      assert.equal(err, null);
      assert.equal(version, 1);
      assert.end();
    });
  });

  test("ChangeStore.getVersion should return version==0 if no changes are found", function(assert) {
    store.getVersion('not-existing-doc', function(err, version) {
      assert.equal(err, null);
      assert.equal(version, 0, 'Document version should equal 0');
      assert.end();
    });
  });


  test("ChangeStore.deleteChanges of test doc", function(assert) {
    store.deleteChanges('test-doc-1', function(err, changeCount) {
      assert.notOk(err, 'Should not error');
      assert.equal(changeCount, 1, 'There should be 1 deleted change');
      store.getChanges({
        documentId: 'test-doc-1',
        sinceVersion: 0
      }, function(err, result) {
        assert.notOk(err, 'Should not error');
        assert.equal(result.changes.length, 0, 'There should not be changes anymore');
        assert.equal(result.version, 0, 'Document version should be 0');
        assert.end();
      });
    });
  });

  test("ChangeStore.deleteChanges of not existing doc", function(assert) {
    store.deleteChanges('not-existing-doc', function(err, changeCount) {
      assert.notOk(err, 'Should not error');
      assert.equal(changeCount, 0, 'There should be 0 deleted changes');
      assert.end();
    });
  });

}
