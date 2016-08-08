'use strict';

var Component = require('substance/ui/Component');
var documentHelpers = require('substance/model/documentHelpers');

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
      
      var props = commandStates[name];
      // Add command name to `props`
      // A necessary hack at time of writing for icons to render in Substance tools
      props.name = name;
      // Add the first selected node of this type to `props`
      props.node = null;
      if (props.active) {
        var session = this.context.documentSession;
        props.node = documentHelpers.getPropertyAnnotationsForSelection(
          session.getDocument(),
          session.getSelection(), {
            type: name
          }
        )[0];
      }

      el.append(
        $$(tool.Class, props).ref(name)
      );

      // An active `Mark` node does not "count" towards enabling the toolbar
      // (because the associated discussion comes up instead)
      if (!(name === 'mark' && props.active)) {
        enabled = enabled || !props.disabled;
      }
    }.bind(this));

    if (enabled) el.addClass('sm-enabled');

    return el;
  };

};

Component.extend(TextToolset);

module.exports = TextToolset;
