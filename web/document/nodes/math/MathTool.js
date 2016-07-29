'use strict';

var Tool = require('substance/ui/Tool');
var documentHelpers = require('substance/model/documentHelpers');

/**
 * A tool used for CRUD on `Math` nodes
 *
 * @class      MathTool (name)
 */
function MathTool() {
  MathTool.super.apply(this, arguments);
}

MathTool.Prototype = function() {

  var _super = MathTool.super.prototype;

  this.render = function($$) {

    // Get details from the currently selected math node (if any)
    var node = null;
    var source = '';
    var language = 'asciimath';
    var display = 'inline'
    if (this.props.active) {
      var session = this.context.documentSession;
      node = documentHelpers.getPropertyAnnotationsForSelection(
        session.getDocument(),
        session.getSelection(), {
          type: 'math'
        }
      )[0];
      source = node.source;
      language = node.language;
      display = node.display;
    }

    console.log('render');

    return $$('div')
      .addClass('se-tool sc-math-tool')
      .addClass('sc-math-tool')
      .append(
        $$('button')
          .ref('language')
          .addClass('se-display')
          .append(
            $$('i')
              .addClass('fa fa-' + (language === 'asciimath' ? 'motorcycle' : 'car'))
          ).on('click', function(event) {
            // Create a node, if necessary, or toggle the language
            if (!node) {
              if (!this.props.disabled) this.performAction();
            } else {
              session.transaction(function(tx) {
                tx.set([node.id, 'language'], language === 'asciimath' ? 'tex' : 'asciimath');
              });
            }
            event.preventDefault();
            event.stopPropagation();
          }),
        $$('div')
          .ref('details')
          .addClass('se-details' + (this.props.active ? ' sm-enabled' : ''))
          .append(
            $$('input')
              .ref('source')
              .addClass('se-source')
              .attr({
                value: source,
                placeholder: 'AsciiMath expression'
              })
              .on('input', function(event) {
                // Update "on-the-fly" (`input` event instead of `change` event) 
                // so user to that user can see live updates of rendered
                // math as they change the source
                session.transaction(function(tx) {
                  tx.set([node.id, 'source'], event.target.value);
                });
                event.preventDefault();
                event.stopPropagation();
              }),
            $$('button')
              .ref('display')
              .addClass('se-display')
              .append(
                $$('i')
                  .addClass('fa fa-' + (display === 'block' ? 'square' : 'minus'))
              )
              .on('click', function(event) {
                // Toggle the display mode between block and inline
                session.transaction(function(tx) {
                  tx.set([node.id, 'display'], display === 'block' ? 'inline' : 'block');
                });
                event.preventDefault();
                event.stopPropagation();
              })
          )
      );
  };

};

Tool.extend(MathTool);

module.exports = MathTool;
