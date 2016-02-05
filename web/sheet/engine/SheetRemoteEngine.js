'use strict';

var oo = require('substance/util/oo');
var RemoteEngine = require('../../RemoteEngine');

function SheetRemoteEngine() {
  SheetRemoteEngine.super.apply(this, arguments);

  window._engine = this;
}

SheetRemoteEngine.Prototype = function() {

  /**
   * Get a list of function names available in the sheet's context
   */
  this.functions = function(cb) {
    this._request('GET', 'functions', null, function(err, result) {
      if (err) return cb(err);
      cb(null, result);
    });
  };

  /**
   * Get a function definition from the sheet's context
   */
  this.function = function(name, cb) {
    this._request('GET', 'function', {name:name}, function(err, result) {
      if (err) return cb(err);
      cb(null, result);
    });
  };

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
