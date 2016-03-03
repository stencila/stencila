'use strict';

var RemoteEngine = require('../../RemoteEngine');

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
    this._request('GET', 'functions', null, function(err, result) {
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
      this._request('PUT', 'function', {name:name}, function(err, result) {
        if (err) return cb(err);
        this._functionSpecs[name] = result;
        cb(null, result);
      }.bind(this));
    }
  };
  
  /*
    Updates given cells
  */
  this.update = function(cells, cb) {
    this._request('PUT', 'update', cells, function(err, result) {
      if (err) return cb(err);
      cb(null, result);
    });
  };

  this.save = function(html, cb) {
  	console.log('TODO: implement save in SheetRemoteEngine.js');
  };
};

RemoteEngine.extend(SheetRemoteEngine);

module.exports = SheetRemoteEngine;
