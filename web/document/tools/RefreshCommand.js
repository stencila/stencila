'use strict';

var Command = require('substance/ui/Command');

function RefreshCommand () {
  RefreshCommand.super.apply(this, arguments);
}

RefreshCommand.Prototype = function () {
  this.getCommandState = function (props, context) {
    return {
      disabled: false,
      active: false
    };
  };

  this.execute = function (props, context) {
    return {
      status: 'render-process-started'
    };
  };
};

Command.extend(RefreshCommand);

module.exports = RefreshCommand;
