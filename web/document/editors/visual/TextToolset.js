'use strict';

var Component = require('substance/ui/Component');

function TextToolset() {
  Component.apply(this, arguments);

  this.tools = [
    'emphasis', 'strong', 'subscript', 'superscript', 'code', 
    'link', 'mark',
    'math', 'print', 'emoji'
  ];
}

TextToolset.Prototype = function() {

  this.render = function($$) {
    var el = $$('div')
      .addClass('sc-toolset sc-text-toolset');

    var enabled = false;
    var toolRegistry = this.context.toolRegistry;
    var commandStates = this.context.commandManager.getCommandStates();
    this.tools.forEach(function(name) {
      var tool = toolRegistry.get(name);
      var state = commandStates[name];
      state.name = name; // A necessary hack at time of writing for icons to render in Substance tools
      el.append(
        $$(tool.Class, state).ref(name)
      );
      // An active `Mark` node does not "count" towards enabling the toolbar
      // (because the associated discussion comes up instead)
      if (!(name === 'mark' && state.active)) {
        enabled = enabled || !state.disabled;
      }
    }.bind(this));

    if (enabled) el.addClass('sm-enabled');

    return el;
  };

};

Component.extend(TextToolset);

module.exports = TextToolset;
