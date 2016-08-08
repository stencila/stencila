'use strict';

var Tool = require('substance/ui/Tool');
var map = require('substance/node_modules/lodash/map');

/**
 * A tool for editing `Math` nodes
 * 
 * Updates the node's `source` property on the `input` event to allow for live updating.
 *
 * @class      MathTool (name)
 */
function MathTool() {
  MathTool.super.apply(this, arguments);
}

MathTool.Prototype = function() {

  var _super = MathTool.super.prototype;

  this.render = function($$) {
    var node = this.props.node;
    var language = node ? node.language : 'asciimath';
    var display = node ? node.display : 'inline';
    return _super.render.call(this, $$)
      .addClass('sc-math-tool')
      .append(
        $$('div')
          .ref('details')
          .addClass('se-details')
          .append(
            $$('input')
              .ref('source')
              .addClass('se-source')
              .attr({
                placeholder: 'Math markup expression',
                title: 'Math markup'
              })
              .val(node ? node.source : null)
              .on('input', function(event) {
                var session = this.context.documentSession;
                session.transaction(function(tx) {
                  tx.set([node.id, 'source'], event.target.value);
                });
              }.bind(this)),
            $$('select')
              .ref('language')
              .addClass('se-language')
              .append(map([['asciimath','AM'],['tex','TeX']], function(item){
                var option = $$('option')
                  .val(item[0])
                  .html(item[1]);
                if (item[0] == language) option.attr('selected', true);
                return option;
              }))
              .on('change', function(event) {
                var session = this.context.documentSession;
                session.transaction(function(tx) {
                  tx.set([node.id, 'language'], event.target.value);
                });
              }.bind(this)),
            $$('select')
              .ref('display')
              .addClass('se-display')
              .append(map([['inline','Inline'],['block','Block']], function(item){
                var option = $$('option')
                  .val(item[0])
                  .html(item[1]);
                if (item[0] == display) option.attr('selected', true);
                return option;
              }))
              .on('change', function(event) {
                var session = this.context.documentSession;
                session.transaction(function(tx) {
                  tx.set([node.id, 'display'], event.target.value);
                });
              }.bind(this))
          )
      );
  };

  this.shouldRerender = function(props) {
    // Do not re-render if the node has not changed.
    // This prevents the input box being updated during live editing
    return (this.props.node === null) || (props.node !== this.props.node);
  };

};

Tool.extend(MathTool);

module.exports = MathTool;
