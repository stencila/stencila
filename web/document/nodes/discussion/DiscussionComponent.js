'use strict';

var IsolatedNodeComponent = require('substance/ui/IsolatedNodeComponent');
var ContainerEditor = require('substance/ui/ContainerEditor');

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
      top: null
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
   * Method override to reflect custom display state
   */
  this.render = function($$) {
    return _super.render.call(this, $$)
      .addClass('sc-discussion ' + (this.state.displayed ? 'sm-displayed' : ''))
      .css('top', (this.state.top ? this.state.top + 'px' : ''))
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
              .append(
                $$('i')
                  .addClass('fa fa-reply')
              )
              .on('click', this.onAddClicked, this)
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
      top: event.detail.markPosition.top
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
    var user = this.context.doc.user;
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

};

IsolatedNodeComponent.extend(DiscussionComponent);

module.exports = DiscussionComponent;
