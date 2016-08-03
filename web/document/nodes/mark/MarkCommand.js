'use strict';

var AnnotationCommand = require('substance/ui/AnnotationCommand');
var uuid = require('substance/util/uuid');

var moment = require('moment');


function MarkCommand() {
  MarkCommand.super.call(this, {
    name: 'mark',
    nodeType: 'mark'
  });
}

MarkCommand.Prototype = function() {

  /**
   * Override `AnnotationCommand.getAnnotationData` to be able to provide
   * a `target` for the mark
   *
   * @return     {Object}  The annotation data.
   */
  this.getAnnotationData = function() {
    return {
      target: uuid('discussion')
    };
  };

  this.execute = function(props, context) {
    var result = MarkCommand.super.prototype.execute.call(this, props, context);
    var mark = result.anno;
    
    // Create a new `Discussion` node after the end of the current selection
    if (result.mode == 'create') {
      var surface = context.surfaceManager.getSurface(props.selection.surfaceId);
      var discussionId;
      surface.transaction(function(tx, args) {
        // Create the new discussion with an initial comment
        var paragraph =  tx.create({
          type: 'paragraph'
        });
        var comment =  tx.create({
          type: 'comment',
          who: '@' + context.doc.user,
          when: moment().format(),
          nodes: [paragraph.id]
        });
        var discussion = tx.create({
          id: mark.target,
          type: 'discussion',
          nodes: [comment.id]
        });
        // Insert the new node after the current one
        var container = tx.get(args.containerId);
        var pos = container.getPosition(args.selection.getNodeId());
        container.show(discussion.id, pos + 1);

        args.node = discussion;
        args.selection = tx.createSelection([paragraph.id, 'content'], 0, 0);

        // CHECK
        // There must be a better way to get the id of the new discussion back
        // from the transaction?
        discussionId = discussion.id;

        return args;
      });

      // CHECK
      // Better way to do this?
      document.dispatchEvent(new CustomEvent('mark:selected', {
        detail: {
          discussionId: discussionId,
          markPosition: {top: '1em'},
        }
      }));

      return true;
    }

    return false;
  }

};


AnnotationCommand.extend(MarkCommand);

module.exports = MarkCommand;
