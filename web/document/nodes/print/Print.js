'use strict';

var InlineNode = require('substance/model/InlineNode');

function Print() {
  Print.super.apply(this, arguments);

  if (!this.content) this.refresh();
}

Print.Prototype = function() {

  this.refresh = function() {
    if (this.source) {
      // TODO
      // Evaluate the source within the document's current
      // context
      try {
        this.content = eval(this.source).toString();
        this.error = false;
      } catch (error) {
        this.content = error.toString();
        this.error = true;
      }
      this.emit('content:changed');
    }
  };

};

InlineNode.extend(Print);

Print.define({
  type: 'print',
  source: { type: 'string', optional: true },
  error: { type: 'boolean', default: false },
  content: { type: 'string', optional: true }
});

module.exports = Print;
