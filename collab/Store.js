'use strict';

import oo from 'substance/util/oo'

var redis = require('redis');

function Store () {
  // TODO Share client across stores
  // TODO Read in a Redis config
  var config = {
    redis: {
      // host
      // port
    }
  };
  this.client = redis.createClient(config.redis);
}

Store.Prototype = function () {
};

oo.initClass(Store);

module.exports = Store;
