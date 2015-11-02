var AnnotationComponent = require('substance/ui/AnnotationComponent');
var StencilNodeComponent = require('../../StencilNodeComponent');
var $$ = require('substance/ui/Component').$$;

var LinkComponent = AnnotationComponent.extend({

  displayName: "LinkComponent",
  
  render: function() {
    var el = AnnotationComponent.prototype.render.call(this);
    var titleComps = [this.props.node.url];
    if (this.props.node.title) {
      titleComps.push(this.props.node.title);
    }

    if (!this.isEditable()) {
      el = $$('a').addClass('link annotation').attr('href', this.props.node.url).append(this.props.children);
    }

    return el.attr("title", titleComps.join(' | '));
  },

  didMount: function() {
    AnnotationComponent.prototype.didMount.call(this);
    var node = this.props.node;
    this.doc = node.getDocument();
    this.doc.getEventProxy('path').add([node.id, 'title'], this, this.rerender);
    this.doc.getEventProxy('path').add([node.id, 'url'], this, this.rerender);
  },

  dispose: function() {
    AnnotationComponent.prototype.dispose.call(this);
    this.doc.getEventProxy('path').remove([this.props.node.id, 'title'], this);
    this.doc.getEventProxy('path').remove([this.props.node.id, 'url'], this);
  }

}, StencilNodeComponent.prototype);

module.exports = LinkComponent;