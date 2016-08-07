'use strict';

var Tool = require('substance/ui/Tool');
var documentHelpers = require('substance/model/documentHelpers');
var insertText = require('substance/model/transform/insertText');

/**
 * A tool used to edit `Emoji` nodes
 *
 * @class      EmojiTool (name)
 */
function EmojiTool() {
  EmojiTool.super.apply(this, arguments);
}

EmojiTool.Prototype = function() {

  var _super = EmojiTool.super.prototype;

  this.render = function($$) {
    var node = null;
    var name = '';
    if (this.props.active) {
      var session = this.context.documentSession;
      node = documentHelpers.getPropertyAnnotationsForSelection(
        session.getDocument(),
        session.getSelection(), {
          type: 'emoji'
        }
      )[0];
      name = node.name;
    }

    return _super.render.call(this, $$)
      .addClass('sc-emoji-tool')
      .append(
        $$('div')
          .ref('details')
          .addClass('se-details')
          .append(
            $$('input')
              .ref('name')
              .addClass('se-name')
              .attr({
                placeholder: 'Emoji name'
              })
              .val(name)
              .on('input', function(event) {
                var session = this.context.documentSession;
                session.transaction(function(tx) {
                  tx.set([node.id, 'name'], event.target.value);
                });
                event.preventDefault();
                event.stopPropagation();
              }.bind(this))
          )
      );
  };

};

Tool.extend(EmojiTool);

module.exports = EmojiTool;
