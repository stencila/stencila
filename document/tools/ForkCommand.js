'use strict';

import Command from 'substance/ui/Command'

function ForkCommand () {
  ForkCommand.super.apply(this, arguments);
}

ForkCommand.Prototype = function () {
  this.getCommandState = function (props, context) {
    return {
      disabled: false,
      active: false
    };
  };

  this.execute = function (props, context) {
    return {
      status: null
    };
  };
};

Command.extend(ForkCommand);

module.exports = ForkCommand;
