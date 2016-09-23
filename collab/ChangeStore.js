import Err from 'substance/util/SubstanceError'

import map from 'lodash/map'

import Store from './Store'

/**
 * Stores changes to Stencila group sessions.
 *
 * Used to sync component group sessions across collaborators.
 *
 * @class      ChangeStore (name)
 */
function ChangeStore () {
  ChangeStore.super.apply(this, arguments)
}

ChangeStore.Prototype = function () {
  /**
   * Add a change and return the new version
   *
   * @param      {<type>}    args    The arguments
   */
  this.addChange = function (args, cb) {
    if (!args.documentId) {
      return cb(new Err('ChangeStore.CreateError', {
        message: 'No documentId provided'
      }))
    }

    if (!args.change) {
      return cb(new Err('ChangeStore.CreateError', {
        message: 'No change provided'
      }))
    }

    // `RPUSH` returns the new size of the list (just what we need!)
    this.client.rpush(args.documentId + ':changes', JSON.stringify(args.change), function (err, size) {
      if (err) return cb(err)
      cb(null, size)
    })
  }

  /*
    Gets changes and latest version for a given document

    @param {String} args.documentId The document id
    @param {Number} args.sinceVersion Since which version
  */
  this.getChanges = function (args, cb) {
    if (!args.sinceVersion) {
      args.sinceVersion = 0
    }
    if (args.sinceVersion < 0) {
      return cb(new Err('ChangeStore.ReadError', {
        message: 'Illegal argument "sinceVersion":' + args.sinceVersion
      }))
    }

    if (args.toVersion < 0) {
      return cb(new Err('ChangeStore.ReadError', {
        message: 'Illegal argument "toVersion":' + args.toVersion
      }))
    }

    if (args.sinceVersion >= args.toVersion) {
      return cb(new Err('ChangeStore.ReadError', {
        message: 'Illegal version arguments "sinceVersion":' + args.sinceVersion + ', "toVersion":' + args.toVersion
      }))
    }

    // `LLEN` return length of list
    this.client.llen(args.documentId + ':changes', function (err, version) {
      if (err) return cb(err)
      // For `toVersion`, use latest version (all changes) or changes UP TO specified version (so
      // a document version can be reconstructed) See `substance/collab/ChangeStore` which uses an
      // array slice for this (which does not include last element)
      var toVersion
      if (args.toVersion) toVersion = args.toVersion - 1
      else toVersion = version
      // `LRANGE` returns and array of strings so JSONize them
      this.client.lrange(args.documentId + ':changes', args.sinceVersion, toVersion, function (err, changes) {
        if (err) return cb(err)
        cb(null, {
          version: version,
          changes: map(changes, JSON.parse)
        })
      })
    }.bind(this))
  }

  /**
   * Get the version (number of changes) for a document
   *
   * @param      {<type>}    documentId      The document identifier
   */
  this.getVersion = function (documentId, cb) {
    // `LLEN` return length of list
    this.client.llen(documentId + ':changes', function (err, version) {
      if (err) return cb(err)
      cb(null, version)
    })
  }

  /**
   * Delete all changes for a document and return the number of
   * changes deleted
   *
   * @param      {<type>}    documentId  The document identifier
   */
  this.deleteChanges = function (documentId, cb) {
    if (!documentId) {
      return cb(new Err('ChangeStore.DeleteError', {
        message: 'No documentId provided'
      }))
    }

    this.client.multi()
      // `LLEN` return length of list
      .llen(documentId + ':changes')
      // `DEL`ete the list
      .del(documentId + ':changes')
      // Return the first reply, the one from `LLEN`
      .exec(function (err, replies) {
        if (err) return cb(err)
        cb(null, replies[0])
      })
  }
}

Store.extend(ChangeStore)

export default ChangeStore
