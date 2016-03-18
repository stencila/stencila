'use strict';

var RemoteEngine = require('../../shared/RemoteEngine');

function SheetRemoteEngine() {
  SheetRemoteEngine.super.apply(this, arguments);
  window._engine = this;
}

SheetRemoteEngine.Prototype = function() {

  /**
    A list of function names currently available in the
    the sheet's context
  */
  this._functionList = [];

  /**
    A dictionary of functions definitions used as 
    a cache
  */
  this._functionSpecs = {};

  /**
    Update the cache of available functions

    TODO: this should be run on app start and when new packages are
          imported that expose more functions.
  */
  this.updateFunctionList = function() {
    this._call('functions', [], function(err, result) {
      this._functionList = result;
    }.bind(this));
  };

  /**
    Gets a list of available functions.
  */
  this.getFunctionList = function() {
    return this._functionList;
  };

  /**
    Get a function specification by name. 
  */
  this.getFunctionSpec = function(name, cb) {
    var cachedFunction = this._functionSpecs[name];
    if (cachedFunction) {
      cb(null, cachedFunction);
    } else {
      this._call('function', [name], function(err, result) {
        if (err) return cb(err);
        this._functionSpecs[name] = result;
        cb(null, result);
      }.bind(this));
    }
  };

  /**
   * Override of `RemoteEngine.boot` which initialises `_functionList`
   */
  this.boot = function(cb) {
    this.super.boot.call(this, function(err, result) {
      if (this.active) this.updateFunctionList();
      if (cb) cb();
    }.bind(this));
  };
  
  /*
    Updates given cells
  */
  this.update = function(cells, cb) {
    this._call('update', [cells], cb);
  };

};

RemoteEngine.extend(SheetRemoteEngine);

module.exports = SheetRemoteEngine;
