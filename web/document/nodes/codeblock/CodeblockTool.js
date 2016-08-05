'use strict';

var BlockTool = require('../../ui/BlockTool');

/**
 * A tool to edit Codeblocks
 * 
 * Just changes `language` (`source` is edited via ACE editor)
 *
 * @class      CodeblockTool (name)
 */
function CodeblockTool() {
  CodeblockTool.super.apply(this, arguments);
}

CodeblockTool.Prototype = function() {

  var _super = CodeblockTool.super.prototype;

  this.render = function($$) {
    var node = this.props.node;
    return _super.render.call(this, $$)
      .addClass('sc-codeblock-tool')
      .append(
        $$('div')
          .ref('details')
          .addClass('se-details')
          .append(
            $$('input')
              .ref('language')
              .attr({
                placeholder: 'Enter the code language',
                spellcheck: 'false'
              })
              .val(node.language)
              .on('change', function(event){
                var surface = this.context.surfaceManager.getFocusedSurface();
                surface.transaction(function(tx, args) {
                  tx.set([node.id, 'language'], event.target.value);
                  return args;
                });
              }.bind(this))
          )
      );
  };

};

BlockTool.extend(CodeblockTool);

module.exports = CodeblockTool;
