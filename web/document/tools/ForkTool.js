'use strict';

var Tool = require('substance/ui/Tool');

function ForkTool () {

  ForkTool.super.apply(this, arguments);

}

ForkTool.Prototype = function () {

  this.getTitle = function () {

    return 'Create a fork of this document; not yet implemented :(';

  };

};

Tool.extend(ForkTool);

module.exports = ForkTool;

