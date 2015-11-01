'use strict';

var oo = require('substance/util/oo');
var Panel = require('substance/ui/Panel');
var Component = require('substance/ui/Component');
var Icon = require('substance/ui/FontAwesomeIcon');
var $$ = Component.$$;

// List existing bib items
// -----------------

function EditSourcePanel() {
  Panel.apply(this, arguments);
}

EditSourcePanel.Prototype = function() {

  this.dispose = function() {
    // TODO: dispose Ace editor instance
  };

  this.didMount = function() {
    // TODO: inject Ace into editor (this.refs.panelContent)
  };

  this.handleCancel = function(e) {
    e.preventDefault();
    this.send('switchContext', 'toc');
  };

  this.render = function() {
    var doc = this.getDocument();
    var node = doc.get(this.props.nodeId);

    console.log('le node', node.source);

    var headerEl = $$('div').addClass('dialog-header').append(
      $$('a').addClass('back').attr('href', '#')
        .on('click', this.handleCancel)
        .append($$(Icon, {icon: 'fa-chevron-left'})),
      $$('div').addClass('label').append('Edit Source')
    );

    var el = $$('div').addClass('sc-edit-source-panel panel dialog');
    var panelContentEl = $$('div').addClass('panel-content').ref('panelContent');
    panelContentEl.append('CODE EDITOR GOES HERE');
    
    el.append(headerEl);
    el.append(panelContentEl);
    return el;
  };
};

oo.inherit(EditSourcePanel, Panel);
module.exports = EditSourcePanel;
