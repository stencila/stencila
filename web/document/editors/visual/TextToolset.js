'use strict';

var Component = require('substance/ui/Component');
var Tool = require('substance/ui/Tool');
var documentHelpers = require('substance/model/documentHelpers');


var Toolset = require('../Toolset');

function TextToolset() {
  Component.apply(this, arguments);

  this.tools = [
    'emphasis', 'strong', 'subscript', 'superscript', 'code', 
    'link', 'math', 'print'
  ];
}

TextToolset.Prototype = function() {

  this.render = function($$) {

    var el = $$('div').addClass('sc-toolset sc-text-toolset');

    var disabled = true;
    this.tools.forEach(function(name) {
      var tool = this.props.toolRegistry.get(name);
      var state = this._getCommandState(name);
      el.append(
        $$(tool.Class, state).ref(name)
      );
      disabled = disabled && state.disabled;
    }.bind(this));

    if (disabled) el.addClass('sm-disabled');

    return el;
  };

};

Toolset.extend(TextToolset);

module.exports = TextToolset;
