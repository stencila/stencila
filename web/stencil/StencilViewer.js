'use strict';

var oo = require('substance/util/oo');
var StencilController = require('./StencilController');
var ContainerAnnotator = require('substance/ui/ContainerAnnotator');
var Component = require('substance/ui/Component');
var Icon = require('substance/ui/FontAwesomeIcon');

var $$ = Component.$$;

var CONFIG = {
  controller: {
    commands: [
      require('substance/ui/UndoCommand'),
      require('substance/ui/RedoCommand'),
      require('substance/ui/SaveCommand'),
    ],
    components: {
      'paragraph': require('substance/packages/paragraph/ParagraphComponent'),
      'heading': require('substance/packages/heading/HeadingComponent'),
      'link': require('./packages/link/LinkComponent'),

      'stencil-exec': require('./packages/exec/StencilExecComponent'),
      'stencil-figure': require('./packages/figure/StencilFigureComponent'),
      'stencil-text': require('./packages/text/StencilTextComponent'),
      'stencil-default-node': require('./packages/default/StencilDefaultNodeComponent'),
    }
  },
  body: {
    commands: [],
  },
  panelOrder: ['toc'],
  containerId: 'body',
  isEditable: false
};

function StencilViewer(parent, params) {
  StencilController.call(this, parent, params);
}

StencilViewer.Prototype = function() {

  this.static = {
    config: CONFIG
  };

  this.toggleReveal = function() {
    this.setState({
      revealSource: !this.state.revealSource
    });
  };

  this.getInitialState = function() {
    return {
      revealSource: true
    };
  };

  this.render = function() {
    var config = this.getConfig();
    var el = $$('div').addClass('sc-stencil-viewer');

    var mainEl = $$('div').addClass('se-main');
    var iconEl;

    if (this.state.revealSource) {
      iconEl = $$(Icon, {icon: 'fa-check-square-o'});
    } else {
      iconEl = $$(Icon, {icon: 'fa-square-o'});
    }

    mainEl.append(
      $$('button').addClass('se-toggle-reveal').append(
        iconEl,
        ' Reveal Source'
      ).on('click', this.toggleReveal)
    );

    mainEl.append(
      $$(ContainerAnnotator, {
        name: 'body',
        containerId: 'body',
        editable: false,
        commands: config.body.commands
      }).ref('bodyAnnotator')
    );

    el.append(mainEl);
    return el;
  };
};

oo.inherit(StencilViewer, StencilController);

module.exports = StencilViewer;
