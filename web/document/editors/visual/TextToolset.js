'use strict';

var Overlay = require('substance/ui/Overlay');
var Tool = require('substance/ui/Tool');

function TextToolset() {
  Overlay.apply(this, arguments);

  this.tools = [
    'emphasis', 'strong', 'subscript', 'superscript', 'code', 
    'link', 'math', 'print'
  ];
}

TextToolset.Prototype = function() {

  this.render = function($$) {
    var el = $$('div').addClass('sc-toolset sc-text-toolset');

    var disabled = true;
    var toolRegistry = this.context.toolRegistry;
    var commandStates = this.context.commandManager.getCommandStates();
    this.tools.forEach(function(name) {
      var tool = toolRegistry.get(name);
      var state = commandStates[name];
      state.name = name; // A necessary hack at time of writing for icons to render in Substance tools
      el.append(
        $$(tool.Class, state).ref(name)
      );
      disabled = disabled && state.disabled;
    }.bind(this));

    if (disabled) el.addClass('sm-disabled');

    return el;
  };

};

Overlay.extend(TextToolset);

module.exports = TextToolset;
