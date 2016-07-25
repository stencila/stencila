'use strict';

var Component = require('substance/ui/Component');

/**
 * A `Toolset` is a set of Substance `Tools`
 * 
 * It is different from a `Toolbar` in that it
 * 
 *  - can be provided a ToolRegistry and a list of tools
 *    to be used from that registry (a Toolbar takes it's
 *    tools from it's context and displays them all)
 *    
 *  - it adds the `sm-disabled` class if all it's tools
 *    are disabled
 *
 * @class      Toolset (name)
 */
function Toolset() {
  Component.apply(this, arguments);
}

Toolset.Prototype = function() {

  this.render = function($$) {
    var disabled = true;
    var tools = [];
    this.props.toolList.forEach(function(name) {
      var tool = this.props.toolRegistry.get(name);
      if (!tool.options.overlay) {
        var state = this.props.commandStates[name];
        disabled = disabled && state.disabled;
        tools.push(
          $$(tool.Class, state)
        );
      }
    }.bind(this));

    var el = $$("div").addClass('sc-toolset');
    if (disabled) el.addClass('sm-disabled');
    el.append(tools);

    return el;
  };

};

Component.extend(Toolset);

module.exports = Toolset;
