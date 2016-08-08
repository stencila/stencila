'use strict';

var oo = require('substance/util/oo');

function Macro () {
};

Macro.Prototype = function() {

  this.appliesTo = [];
  
  this.execute = function(props, context) {
    if (!this.regex) {
      throw new Error('Must define `this.regex` for Macro class');
    }

    if (this.appliesTo.length > 0 && this.appliesTo.indexOf(props.node.type) === -1) {
      return false;
    }

    var match = this.regex.exec(props.text);
    if (match) {
      this.performAction(match, props, context);
      return true;
    }

    return false;
  }

  /**
   * Perform the macro action when matched
   *
   * @param      {<type>}  match   The match
   */
  this.performAction = function(match, props, context) {
    throw new Error('This method is abstract.');
  };

};

oo.initClass(Macro);

module.exports = Macro;
