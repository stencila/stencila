'use strict';

var Tool = require('substance/ui/Tool');

/**
 * A tool for editing `Emoji` nodes
 *
 * Updates the node `name` property on the `input` event to allow for live updating.
 *
 * @class      EmojiTool (name)
 */
function EmojiTool () {

  EmojiTool.super.apply(this, arguments);

}

EmojiTool.Prototype = function () {

  var _super = EmojiTool.super.prototype;

  this.render = function ($$) {

    var node = this.props.node;
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
                placeholder: 'Emoji name',
                title: 'Name of emoji'
              })
              .val(node ? node.name : null)
              .on('input', function (event) {

                var session = this.context.documentSession;
                session.transaction(function (tx) {

                  tx.set([node.id, 'name'], event.target.value);

                });

              }.bind(this))
          )
      );

  };

  this.shouldRerender = function (props) {

    // Do not re-render if the node has not changed.
    // This prevents the input box being updated during live editing
    return (this.props.node === null) || (props.node !== this.props.node);

  };

};

Tool.extend(EmojiTool);

module.exports = EmojiTool;
