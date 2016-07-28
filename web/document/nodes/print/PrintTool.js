'use strict';

var AnnotationTool = require('substance/ui/AnnotationTool');
var documentHelpers = require('substance/model/documentHelpers');

/**
 * A tool used for CRUD on `Print` nodes
 *
 * @class      PrintTool (name)
 */
function PrintTool() {
  PrintTool.super.apply(this, arguments);
}

PrintTool.Prototype = function() {

  var _super = PrintTool.super.prototype;

  this.render = function($$) {
    var el = _super.render.call(this, $$)
      .addClass('sc-print-tool');

    var source = null;
    if (this.props.active) {
      var session = this.context.documentSession;
      var print = documentHelpers.getPropertyAnnotationsForSelection(session.getDocument(), session.getSelection(), {
        type: 'print'
      })[0];
      source = print.source;
    }

    // Render details even if not active so that expansion
    // animation works 
    var details = $$('span')
      .addClass('se-details')
      .ref('details')
      .append(
        $$('input')
          .attr({
            value: source,
            placeholder: 'A host language expression'
          })
          .on('change', function(event){
            session.transaction(function(tx) {
              tx.set([print.id, 'source'], event.target.value);
            });
          })
      );
    if (this.props.active) details.addClass('sm-enabled');
    el.append(details);

    return el;
  };

};

AnnotationTool.extend(PrintTool);

module.exports = PrintTool;
