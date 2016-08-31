'use strict';

var Command = require('substance/ui/Command');

function SettingsCommand () {

  SettingsCommand.super.apply(this, arguments);

}

SettingsCommand.Prototype = function () {

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

Command.extend(SettingsCommand);

module.exports = SettingsCommand;
