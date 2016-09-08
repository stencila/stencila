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
    var doc = context.doc;
    var annotations = doc.getIndex('annotations');
    var refresh = function (node) {
      if (node.refresh) {
        node.refresh();
      }
      if (node.hasChildren()) {
        node.getChildren().forEach(function (child) {
          refresh(child);
        });
      } else if (node.isText()) {
        annotations.get(node.getTextPath()).forEach(function (child) {
          refresh(child);
        });
      }
    };
    refresh(doc.get('content'));
    return true;
  };
};

Command.extend(RefreshCommand);

module.exports = RefreshCommand;
