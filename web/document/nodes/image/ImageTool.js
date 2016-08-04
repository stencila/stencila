'use strict';

var BlockTool = require('../../ui/BlockTool');

/**
 * A tool to edit images
 *
 * @class      ImageTool (name)
 */
function ImageTool() {
  ImageTool.super.apply(this, arguments);
}

ImageTool.Prototype = function() {

  var _super = ImageTool.super.prototype;

  this.render = function($$) {
    // For placeholder to work override Substance's
    // default for src
    var src = this.props.node.src;
    if (src === 'http://') src = '';
    return _super.render.call(this, $$)
      .addClass('sc-image-tool')
      .append(
        $$('div')
          .ref('details')
          .addClass('se-details')
          .append(
            $$('input')
              .attr({
                value: src,
                placeholder: 'Paste or type a URL',
                spellcheck: 'false'
              })
              .on('change', function(event){
                var session = this.context.documentSession;
                var nodeId = this.props.node.id;
                session.transaction(function(tx) {
                  tx.set([nodeId, 'src'], event.target.value);
                });
              }.bind(this))
          )
      );
  };

};

BlockTool.extend(ImageTool);

module.exports = ImageTool;
