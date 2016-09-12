'use strict';

var Component = require('substance/ui/Component');
var documentHelpers = require('substance/model/documentHelpers');

function TextToolset () {
  Component.apply(this, arguments);

  this.tools = [
    'emphasis', 'strong', 'subscript', 'superscript', 'code',
    'link', 'mark',
    'math', 'print', 'emoji'
  ];

  this.inlineNodeTools = [
    'math', 'print', 'emoji'
  ];
}

TextToolset.Prototype = function () {
  this.render = function ($$) {
    var el = $$('div')
      .addClass('sc-toolset sc-text-toolset');

    var enabled = false;
    var toolRegistry = this.context.toolRegistry;
    var commandStates = this.context.commandManager.getCommandStates();

    this.tools.forEach(function (name) {
      var tool = toolRegistry.get(name);
      var session = this.context.documentSession;
      var sel = session.getSelection();

      var props = commandStates[name];
      // Don't enable `InlineNodeTools` if there is no selected text
      if (!props.disabled && (this.inlineNodeTools.indexOf(name) > -1) && sel && !sel.isNull() && sel.isPropertySelection()) {
        if (sel.getStartOffset() === sel.getEndOffset()) {
          props.disabled = true;
        }
      }
      // Add command name to `props` (a necessary hack at time of writing for icons to render in Substance tools)
      props.name = name;
      // Add the first selected node of this type to `props`
      props.node = null;
      if (props.active) {
        props.node = documentHelpers.getPropertyAnnotationsForSelection(
          session.getDocument(),
          sel,
          {type: name}
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
