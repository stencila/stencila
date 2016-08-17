'use strict';

var oo = require('substance/util/oo');

var redis = require('redis');

function Store() {
  this.client = redis.createClient();
}

Store.Prototype = function() {
};

oo.initClass(Store);

module.exports = Store;
