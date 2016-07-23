'use strict';

var ControllerCommand = require('substance-fe0ed/ui/ControllerCommand');

/**
 * Commit the sheet
 */
function CommitCommand() {
  CommitCommand.super.apply(this, arguments);
}

CommitCommand.Prototype = function() {

  this.getCommandState = function() {
    return {
      disabled: false,
      active: false
    };
  },

  this.execute = function(message, callback) {
    var engine = this.controller.props.engine;
    engine.commit(message, callback);
    return {
      status: 'commit-requested'
    };
  };
};

ControllerCommand.extend(CommitCommand);
CommitCommand.static.name = 'commit';

module.exports = CommitCommand;
