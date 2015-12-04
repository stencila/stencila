'use strict';

var _ = require('substance/util/helpers');

module.exports = {

  updateGeneratedProperties : function(props) {
    var propNames = this.constructor.static.generatedProps;
    if (propNames) {
      _.each(propNames, function(propName) {
        this[propName] = props[propName];
      }, this);
      this.emit('properties:changed');
    }
  },

  setSource : function(source) {
    this.source = source;
    this.emit('source:changed');
  }

};
