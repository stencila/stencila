'use strict';

var IsolatedNodeComponent = require('substance/ui/IsolatedNodeComponent');
var ContainerEditor = require('substance/ui/ContainerEditor');
var deleteNode = require('substance/model/transform/deleteNode');
var each = require('substance/node_modules/lodash/each');

var moment = require('moment');


function DiscussionComponent() {
  DiscussionComponent.super.apply(this, arguments);

  this.ContentClass = ContainerEditor;

  document.addEventListener('mark:selected', this.onMarkSelected.bind(this));
}

DiscussionComponent.Prototype = function() {

  var _super = DiscussionComponent.super.prototype;

  /**
   * Method override for custom display state
   */
  this.getInitialState = function() {
    return {
      displayed: false,
      markPosition: null
    }
  }

  /**
   * Method override so no blocker is rendered over this
   * `IsolatedNodeComponent` (requires two clicks to begin editing)
   */
  this.shouldRenderBlocker = function() {
    return false;
  }

  /**
   * Method override to render component
   */
  this.render = function($$) {
    // Calculate postion based on the size of the margin for the
    // document content. Calculations done using ems assuming 16px em size.
    var content = document.querySelector('.content');
    var em = 16;
    var position, top, left, right;
    if (content) {
      var rect = content.getBoundingClientRect();
      var margin = parseInt(window.getComputedStyle(content).getPropertyValue('margin-right').match(/\d+/));
      if (margin >= 20*em) {
        position = 'fixed';
        top = em + 'px';
        left = rect.right + 'px';
        right = 'inherit';
      } else {
        position = 'absolute';
        if (this.state.markPosition) {
          top = this.state.markPosition.top + this.state.markPosition.height + em + 'px';
        } else {
          top = 'inherit';
        }
        left = 'inherit';
        right = margin + 'px';
      }
    } else {
      // Fallback to top-right
      position = 'fixed';
      top = em + 'px';
      right = em + 'px';
    }
    return _super.render.call(this, $$)
      .addClass('sc-discussion ' + (this.state.displayed ? 'sm-displayed' : ''))
      .css({
        position: position,
        top: top,
        left: left,
        right: right
      })
      .insertAt(0,
        $$('div')
          .ref('header')
          .addClass('se-header')
          .attr('contenteditable', false)
          .append(
            $$('button')
              .ref('icon')
              .addClass('se-icon')
              .append(
                $$('i')
                  .addClass('fa fa-comments-o')
              ),
            $$('button')
              .ref('hide')
              .addClass('se-hide')
              .append(
                $$('i')
                  .addClass('fa fa-close')
              )
              .on('click', this.onHideClicked, this)
          )
      )
    .append(
        $$('div')
          .ref('footer')
          .addClass('se-footer')
          .attr('contenteditable', false)
          .append(
            $$('button')
              .ref('add')
              .addClass('se-add')
              .attr('title', 'Add comment to discussion')
              .append(
                $$('i')
                  .addClass('fa fa-reply')
              )
              .on('click', this.onAddClicked, this),
            $$('button')
              .ref('delete')
              .addClass('se-delete')
              .attr('title', 'Delete discussion')
              .append(
                $$('i')
                  .addClass('fa fa-trash')
              )
              .on('click', this.onDeleteClicked, this)
          )
    );
  }


  /**
   * Event method to change display state any
   * mark is selected
   *
   * @param      {<type>}  event   The event
   */
  this.onMarkSelected = function(event) {
    this.extendState({
      displayed: event.detail.discussionId == this.props.node.id,
      markPosition: event.detail.markPosition
    });
  }

  /**
   * Event method for when the hide button
   * is clicked.
   */
  this.onHideClicked = function() {
    this.extendState({
      displayed: false
    });
  }

  /**
   * Event method for when the add buttin
   * is clicked.
   */
  this.onAddClicked = function() {
    var discussion = this.props.node;
    var user = this.context.documentSession.config.user;
    var surface = this.context.surfaceManager.getFocusedSurface();
    surface.transaction(function(tx, args) {
      // Create a new comment
      var paragraph =  tx.create({
        type: 'paragraph'
      });
      var comment =  tx.create({
        type: 'comment',
        who: '@' + user,
        when: moment().format(),
        nodes: [paragraph.id]
      });
      // Append to the end of the discussion
      discussion.show(comment.id);

      // FIXME
      // The first time the add button is clicked the new comment appears but
      // seems like the whole discussion is selected. Then subsequent clicks
      // don't add anything. Seems to be reated to selection.
      console.warn('FIXME');

      args.node = paragraph;
      args.selection = tx.createSelection([paragraph.id, 'content'], 0, 0);

      return args;
    });

  }

  /**
   * Event method for deleting this discussion and associated `Mark`
   */
  this.onDeleteClicked = function() {
    var discussion = this.props.node;
    var session = this.context.documentSession;
    // Destroy this component first
    this.remove();
    session.transaction(function(tx, args) {

      // Delete the discussion and associated mark
      deleteNode(tx, { nodeId: discussion.id });
      each(session.doc.getNodes(), function(node) {
        if (node.type === 'mark' && node.target === discussion.id) {
          deleteNode(tx, { nodeId: node.id });
        }
      });
      // Return a null selection
      args.selection = tx.createSelection(null);
      return args;

    }.bind(this));
  }

};

IsolatedNodeComponent.extend(DiscussionComponent);

module.exports = DiscussionComponent;
