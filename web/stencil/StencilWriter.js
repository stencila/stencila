'use strict';

var oo = require('substance/util/oo');
var StencilController = require('./StencilController');
var docHelpers = require('substance/model/documentHelpers');
var WriterTools = require('./packages/writer/WriterTools');
var Component = require('substance/ui/Component');
var $$ = Component.$$;

var CONFIG = {
  controller: {
    commands: [
      require('substance/ui/UndoCommand'),
      require('substance/ui/RedoCommand'),
      require('substance/ui/SaveCommand'),
      require('./packages/writer/RenderCommand'),
    ],
    components: {
      'paragraph': require('substance/packages/paragraph/ParagraphComponent'),
      'heading': require('substance/packages/heading/HeadingComponent'),
      'link': require('./packages/link/LinkComponent'),

      'stencil-title': require('./packages/title/StencilTitleComponent'),
      'stencil-summary': require('./packages/summary/StencilSummaryComponent'),

      'stencil-math': require('./packages/math/StencilMathComponent'),
      'stencil-equation': require('./packages/equation/StencilEquationComponent'),

      'stencil-exec': require('./packages/exec/StencilExecComponent'),
      'stencil-figure': require('./packages/figure/StencilFigureComponent'),
      'stencil-text': require('./packages/text/StencilTextComponent'),

      'stencil-default-node': require('./packages/default/StencilDefaultNodeComponent'),

      // Panels
      'toc': require('substance/ui/TOCPanel'),
      'cila': require('./packages/writer/CilaPanel'),
      'edit-source': require('./packages/writer/EditSourcePanel')
    }
  },
  body: {
    commands: [
      // Special commands
      require('substance/packages/embed/EmbedCommand'),
      require('substance/packages/text/SwitchTextTypeCommand'),
      require('substance/packages/strong/StrongCommand'),
      require('substance/packages/emphasis/EmphasisCommand'),
      require('substance/packages/link/LinkCommand'),
      require('./packages/table/InsertTableCommand'),
    ],
    textTypes: [
      {name: 'paragraph', data: {type: 'paragraph'}},
      {name: 'title', data: {type: 'stencil-title'}},
      {name: 'summary', data: {type: 'stencil-summary'}},
      {name: 'heading1',  data: {type: 'heading', level: 1}},
      {name: 'heading2',  data: {type: 'heading', level: 2}},
      {name: 'heading3',  data: {type: 'heading', level: 3}}
    ]
  },
  panels: {
    'toc': {
      hideContextToggles: false
    },
    'cila': {
      hideContextToggles: false
    },
    'edit-source': {
      hideContextToggles: true
    }
  },
  tabOrder: ['toc','cila'],
  containerId: 'body',
  isEditable: false
};

function StencilWriter() {
  StencilWriter.super.apply(this, arguments);
}

StencilWriter.Prototype = function() {

  var _super = Object.getPrototypeOf(this);

  this._renderToolbar = function() {
    return _super._renderToolbar.call(this).append(
      $$(WriterTools)
    );
  };

  this.onSelectionChanged = function(sel, surface) {
    var config = this.getConfig();

    function getActiveAnno(type) {
      return docHelpers.getAnnotationsForSelection(doc, sel, type, config.containerId)[0];
    }

    if (surface.name !== config.containerId) return;
    if (sel.isNull() || !sel.isPropertySelection()) {
      return;
    }
    if (sel.equals(this.prevSelection)) {
      return;
    }
    this.prevSelection = sel;
    var doc = surface.getDocument();

    // Annotation
    var stencilText = getActiveAnno('stencil-text');
    if (stencilText && stencilText.getSelection().equals(sel)) {
      // Trigger state change
      this.setState({
        contextId: 'edit-source',
        nodeId: stencilText.id
      });
    } else {
      if (this.state.contextId !== 'toc') {
        this.setState({
          contextId: 'toc'
        });
      }
    }
  };
};

oo.inherit(StencilWriter, StencilController);

StencilWriter.static.config = CONFIG;

module.exports = StencilWriter;
