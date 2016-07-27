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
        var state = this._getCommandState(name);
        tools.push(
          $$(tool.Class, state)
        );
        disabled = disabled && state.disabled;
      }
    }.bind(this));

    var el = $$("div").addClass('sc-toolset');
    if (disabled) el.addClass('sm-disabled');
    if (this.props.top) el.setStyle('top', this.props.top +'px');
    el.append(tools);

    return el;
  };

  // Private methods

  this._getCommandState = function(name){
      var state = this.props.commandStates[name];
      if (!state) throw new Error('Command {' + name + '} not found');
      state.name = name; // A necessary hack at time of writing
      return state;
  }

};

Component.extend(Toolset);

module.exports = Toolset;
