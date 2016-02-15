'use strict';

var oo = require('substance/util/oo');
var RemoteEngine = require('../../RemoteEngine');

function SheetRemoteEngine() {
  SheetRemoteEngine.super.apply(this, arguments);

  window._engine = this;
}

SheetRemoteEngine.Prototype = function() {

  /**
   * A list of function names currently available in the
   * the sheet's context
   */
  this._functionList = null;

  /**
   * A dictionary of functions definitions used as 
   * a cache
   */
  this._functionSpecs = {}

  /**
   * Get a list of function names
   */
  this.functions = function(cb) {
    if(this._functionList) {
      cb(this._functionList);
    } else {
      this._request('GET', 'functions', null, function(err, result) {
        this._functionList = result;
        cb(result);
      }.bind(this));
    }
  };

  /**
   * Get a function definition
   */
  this.function = function(name, cb) {
    if(this._functionSpecs[name]){
      return this._functionSpecs[name];
    } else {
      this._request('PUT', 'function', {name:name}, function(err, result) {
        this._functionSpecs[name] = result;
        cb(result);
      }.bind(this));
    }
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
