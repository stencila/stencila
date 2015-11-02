'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var TextProperty = require('substance/ui/TextPropertyComponent');
var Icon = require('substance/ui/FontAwesomeIcon');
var $$ = Component.$$;
var StencilNodeComponent = require('../../StencilNodeComponent');

function StencilFigureComponent() {
  StencilNodeComponent.apply(this, arguments);
}

StencilFigureComponent.Prototype = function() {

  this.render = function() {
    var el = $$('div')
      .addClass("sc-stencil-figure content-node figure clearfix "+this.props.node.type)
      .attr("data-id", this.props.node.id)
      .attr('id', this.props.node.id);

    el.append($$('div')
      .addClass('label').attr("contenteditable", false)
      .append(this.props.node.label)
      .key('label')
    );

    if (this.isEditable()) {
      el.append(
        $$('button')
          .addClass('se-figure-edit-button')
          .append(
            $$('span').addClass('se-action').append(
              $$(Icon, {icon: 'fa-pencil'}),
              ' ',
              this.i18n.t('edit-source-action')
            )
          )
          .on('click', this.onClickEdit)
          // Unfortunately we need to suppress mouse down, as otherwise
          // Surface will receive events leading to updating the selection
          .on('mousedown', this.onMouseDown)
      );
    }

    if (this.revealSource()) {
      el.append($$('div')
        .addClass('source')
        .append(
          $$(TextProperty, {
            tagName: 'div',
            path: [this.props.node.id, "source"]
          })
        )
        .key('source')
      );
    }

    el.append($$('div')
      .addClass('figure-content')
      .attr('contenteditable', false)
      .append(
        $$('img')
          .addClass('image')
          .attr({
            contentEditable: false,
            src: this.props.node.getDocument().url + "/" + this.props.node.image
          })
      )
      .key('content')
    );
    el.append($$('div')
      .addClass('description small')
      .append(
        $$(TextProperty, {
          tagName: 'div',
          path: [this.props.node.id, "caption"]
        })
        .addClass('caption')
      )
      .key('description')
    );

    if (this.props.node.error) {
      el.addClass('sm-error');
    }
    return el;
  };

  this.onClickEdit = function(e) {
    e.preventDefault();
    e.stopPropagation();
    this.send('switchState', {
      contextId: 'editSource',
      nodeId: this.props.node.id
    });
  };

  this.onMouseDown = function(e) {
    e.preventDefault();
    e.stopPropagation();
  };

};

oo.inherit(StencilFigureComponent, StencilNodeComponent);

module.exports = StencilFigureComponent;
