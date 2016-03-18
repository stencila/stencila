'use strict';

module.exports = {

  updateGeneratedProperties : function(props) {
    var propNames = this.constructor.static.generatedProps;
    if (propNames) {
      propNames.forEach(function(propName) {
        this[propName] = props[propName];
      }.bind(this));
      this.emit('properties:changed');
    }
  },

  setSource : function(source) {
    this.source = source;
    this.emit('source:changed');
  }

};
