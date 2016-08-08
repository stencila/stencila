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

  /**
   * Create an object with the data for the new node
   * 
   * Should be overidden by derived classes.
   * Analagous to the method with the same name
   * in `substance/ui/InlineNodeCommand`.
   *
   * @param      {<type>}  match   The match
   */
  this.createNodeData = function(match) {
    throw new Error('This method is abstract.');
  };

};

oo.initClass(Macro);

module.exports = Macro;
