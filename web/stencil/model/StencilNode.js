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

  getSource : function() {
    var directive = this.constructor.static.directive;
    if (directive) {
      return {
        lang: 'cila',
        code: directive + ' ' + this.source
      };
    } else {
      return {
        lang: 'text',
        code: this.source
      };
    }
  },

  setSource : function(source) {
    var directive = this.constructor.static.directive;
    if (directive) {
      var matches = source.match('^' + directive + ' (.+)');
      if (matches) {
        source = matches[1];
      }
    }
    this.source = source;
    this.emit('source:changed');
  }

};
