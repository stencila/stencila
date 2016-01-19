'use strict';

var oo = require('substance/util/oo');
var RemoteEngine = require('../../RemoteEngine');

function SheetRemoteEngine() {
  SheetRemoteEngine.super.apply(this, arguments);
}

SheetRemoteEngine.Prototype = function() {

  this.update = function(cells, cb) {
    this.request('PUT', 'update', cells, function(err, result) {
      if (err) { console.error(err); cb(err); }
      cb(null, result);
    });
  };

  this.save = function(html, cb) {
  	console.log('TODO: implement save in SheetRemoteEngine.js');
  };

};

RemoteEngine.extend(SheetRemoteEngine);

module.exports = SheetRemoteEngine;
